use std::future::Future;
use std::io;
use std::io::Cursor;
use std::pin::Pin;
use std::time::Duration;

use futures::stream::unfold;
use futures::Stream;
use futures::StreamExt;
use openraft::entry::RaftEntry;
use openraft::network::Backoff;
use openraft::network::NetBackoff;
use openraft::network::NetSnapshot;
use openraft::network::NetStreamAppend;
use openraft::network::NetTransferLeader;
use openraft::network::NetVote;
use openraft::network::RPCOption;
use openraft::network::RaftNetworkFactory;
use openraft::raft::AppendEntriesRequest;
use openraft::raft::AppendEntriesResponse;
use openraft::raft::SnapshotResponse;
use openraft::raft::StreamAppendResult;
use openraft::raft::TransferLeaderRequest;
use openraft::raft::VoteRequest;
use openraft::raft::VoteResponse;
use openraft::type_config::alias::NodeIdOf;
use openraft::type_config::alias::SnapshotMetaOf;
use openraft::type_config::alias::SnapshotOf;
use openraft::type_config::alias::VoteOf;
use openraft::BasicNode;
use openraft::OptionalSend;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;

use super::raft_node::TypeName;

pub struct HttpRaftNetwork {
    timeout: Duration,
    raft_secret: String,
}

impl HttpRaftNetwork {
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            raft_secret: String::new(),
        }
    }

    pub fn with_secret(timeout: Duration, raft_secret: String) -> Self {
        Self {
            timeout,
            raft_secret,
        }
    }
}

pub struct HttpRaftConn {
    addr: String,
    client: Client,
    raft_secret: String,
}

fn rpc_unreachable(e: impl std::fmt::Display) -> openraft::error::RPCError<TypeName> {
    openraft::error::RPCError::Unreachable(openraft::error::Unreachable::new(&io::Error::new(
        io::ErrorKind::NotConnected,
        e.to_string(),
    )))
}

fn streaming_unreachable(e: impl std::fmt::Display) -> openraft::error::StreamingError<TypeName> {
    openraft::error::StreamingError::Unreachable(openraft::error::Unreachable::new(
        &io::Error::new(io::ErrorKind::NotConnected, e.to_string()),
    ))
}

impl RaftNetworkFactory<TypeName> for HttpRaftNetwork {
    type Network = HttpRaftConn;

    async fn new_client(&mut self, _target: NodeIdOf<TypeName>, node: &BasicNode) -> Self::Network {
        HttpRaftConn {
            addr: node.addr.clone(),
            client: Client::builder()
                .timeout(self.timeout)
                .build()
                .unwrap_or_default(),
            raft_secret: self.raft_secret.clone(),
        }
    }
}

impl NetBackoff<TypeName> for HttpRaftConn {
    fn backoff(&self) -> Backoff {
        Backoff::new(std::iter::repeat(Duration::from_millis(500)))
    }
}

impl NetVote<TypeName> for HttpRaftConn {
    async fn vote(
        &mut self,
        rpc: VoteRequest<TypeName>,
        _option: RPCOption,
    ) -> Result<VoteResponse<TypeName>, openraft::error::RPCError<TypeName>> {
        let url = format!("http://{}/raft/vote", self.addr);
        let resp = self
            .client
            .post(&url)
            .header("x-raft-secret", &self.raft_secret)
            .json(&rpc)
            .send()
            .await
            .map_err(rpc_unreachable)?;
        let vote_resp = resp
            .json::<VoteResponse<TypeName>>()
            .await
            .map_err(rpc_unreachable)?;
        Ok(vote_resp)
    }
}

type AppendStream = Pin<Box<dyn Stream<Item = AppendEntriesRequest<TypeName>> + Send>>;

impl NetStreamAppend<TypeName> for HttpRaftConn {
    fn stream_append<'s, S>(
        &'s mut self,
        input: S,
        _option: RPCOption,
    ) -> openraft::base::BoxFuture<
        's,
        Result<
            openraft::base::BoxStream<
                's,
                Result<StreamAppendResult<TypeName>, openraft::error::RPCError<TypeName>>,
            >,
            openraft::error::RPCError<TypeName>,
        >,
    >
    where
        S: Stream<Item = AppendEntriesRequest<TypeName>> + OptionalSend + Unpin + 'static,
    {
        let client = self.client.clone();
        let addr = self.addr.clone();
        let raft_secret = self.raft_secret.clone();
        let boxed: AppendStream = Box::pin(input);

        let stream = unfold(
            (boxed, client, addr, raft_secret),
            |(mut input, client, addr, raft_secret)| async move {
                let req = match input.next().await {
                    Some(r) => r,
                    None => return None,
                };
                let prev = req.prev_log_id;
                let last = req.entries.last().map(|e| e.log_id()).or(prev);
                let url = format!("http://{}/raft/append", addr);
                let result = match client
                    .post(&url)
                    .header("x-raft-secret", &raft_secret)
                    .json(&req)
                    .send()
                    .await
                {
                    Ok(resp) => match resp.json::<AppendEntriesResponse<TypeName>>().await {
                        Ok(ae_resp) => Ok(ae_resp.into_stream_result(prev, last)),
                        Err(e) => Err(rpc_unreachable(e)),
                    },
                    Err(e) => Err(rpc_unreachable(e)),
                };
                Some((result, (input, client, addr, raft_secret)))
            },
        );

        Box::pin(async move { Ok(Box::pin(stream) as _) })
    }
}

#[derive(Serialize, Deserialize)]
pub struct SnapshotTransport {
    pub vote: VoteOf<TypeName>,
    pub meta: SnapshotMetaOf<TypeName>,
    pub data: Vec<u8>,
}

impl SnapshotTransport {
    pub fn from_parts(vote: VoteOf<TypeName>, snapshot: SnapshotOf<TypeName>) -> Self {
        Self {
            vote,
            meta: snapshot.meta,
            data: snapshot.snapshot.into_inner(),
        }
    }

    pub fn into_parts(self) -> (VoteOf<TypeName>, SnapshotOf<TypeName>) {
        let snap = SnapshotOf::<TypeName> {
            meta: self.meta,
            snapshot: Cursor::new(self.data),
        };
        (self.vote, snap)
    }
}

impl NetSnapshot<TypeName> for HttpRaftConn {
    async fn full_snapshot(
        &mut self,
        vote: VoteOf<TypeName>,
        snapshot: SnapshotOf<TypeName>,
        _cancel: impl Future<Output = openraft::error::ReplicationClosed> + OptionalSend + 'static,
        _option: RPCOption,
    ) -> Result<SnapshotResponse<TypeName>, openraft::error::StreamingError<TypeName>> {
        let url = format!("http://{}/raft/snapshot", self.addr);
        let body = SnapshotTransport::from_parts(vote, snapshot);
        let resp = self
            .client
            .post(&url)
            .header("x-raft-secret", &self.raft_secret)
            .json(&body)
            .send()
            .await
            .map_err(streaming_unreachable)?;
        let snap_resp = resp
            .json::<SnapshotResponse<TypeName>>()
            .await
            .map_err(streaming_unreachable)?;
        Ok(snap_resp)
    }
}

impl NetTransferLeader<TypeName> for HttpRaftConn {
    async fn transfer_leader(
        &mut self,
        req: TransferLeaderRequest<TypeName>,
        _option: RPCOption,
    ) -> Result<(), openraft::error::RPCError<TypeName>> {
        let url = format!("http://{}/raft/transfer", self.addr);
        self.client
            .post(&url)
            .header("x-raft-secret", &self.raft_secret)
            .json(&req)
            .send()
            .await
            .map_err(rpc_unreachable)?
            .json::<()>()
            .await
            .map_err(rpc_unreachable)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_raft_network_default_timeout() {
        let net = HttpRaftNetwork::new(std::time::Duration::from_secs(10));
        assert_eq!(net.timeout, std::time::Duration::from_secs(10));
    }
}
