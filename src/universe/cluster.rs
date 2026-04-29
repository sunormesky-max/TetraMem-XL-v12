use std::collections::BTreeMap;
use std::future::Future;
use std::sync::Arc;

use openraft::async_runtime::watch::WatchReceiver;

use futures::Stream;
use openraft::network::Backoff;
use openraft::network::NetBackoff;
use openraft::network::NetSnapshot;
use openraft::network::NetStreamAppend;
use openraft::network::NetTransferLeader;
use openraft::network::NetVote;
use openraft::network::RaftNetworkFactory;
use openraft::network::RPCOption;
use openraft::type_config::alias::NodeIdOf;
use openraft::type_config::alias::SnapshotOf;
use openraft::type_config::alias::VoteOf;
use openraft::BasicNode;
use openraft::Config;
use openraft::OptionalSend;
use openraft::Raft;
use serde::{Deserialize, Serialize};

use super::raft_node::{new_log_store, new_state_machine, LogStore, Request, StateMachineStore, TypeName};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNodeInfo {
    pub node_id: u64,
    pub addr: String,
    pub role: String,
    pub is_leader: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatus {
    pub node_id: u64,
    pub leader_id: Option<u64>,
    pub nodes: Vec<ClusterNodeInfo>,
    pub term: u64,
    pub log_index: u64,
    pub applied_count: usize,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitClusterRequest {
    pub node_id: u64,
    pub addr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddNodeRequest {
    pub node_id: u64,
    pub addr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveNodeRequest {
    pub node_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposeRequest {
    pub action: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposeResponse {
    pub success: bool,
    pub log_index: u64,
    pub data: serde_json::Value,
    pub conservation_verified: bool,
}

type RaftNode = Raft<TypeName, StateMachineStore>;

pub type ConservationValidator = Box<dyn Fn() -> bool + Send + Sync>;

pub struct ClusterManager {
    node_id: u64,
    raft: Option<RaftNode>,
    log_store: LogStore,
    state_machine: StateMachineStore,
    addr: String,
    peers: BTreeMap<u64, String>,
    conservation_validator: Option<ConservationValidator>,
}

impl ClusterManager {
    pub fn new(node_id: u64, addr: String) -> Self {
        Self {
            node_id,
            raft: None,
            log_store: new_log_store(),
            state_machine: new_state_machine(),
            addr,
            peers: BTreeMap::new(),
            conservation_validator: None,
        }
    }

    pub fn set_conservation_validator(&mut self, v: ConservationValidator) {
        self.conservation_validator = Some(v);
    }

    pub async fn init_single_node(&mut self) -> Result<(), String> {
        let config = Config {
            heartbeat_interval: 500,
            election_timeout_min: 1500,
            election_timeout_max: 3000,
            ..Default::default()
        };
        let config = Arc::new(
            config
                .validate()
                .map_err(|e| format!("raft config error: {}", e))?,
        );

        let raft: RaftNode = Raft::new(
            self.node_id,
            config,
            DummyNetwork,
            self.log_store.clone(),
            self.state_machine.clone(),
        )
        .await
        .map_err(|e| format!("raft init error: {}", e))?;

        let mut nodes = BTreeMap::new();
        let node = BasicNode::new(&self.addr);
        nodes.insert(self.node_id, node);

        raft.initialize(nodes)
            .await
            .map_err(|e| format!("raft initialize error: {}", e))?;

        self.raft = Some(raft);
        self.peers.insert(self.node_id, self.addr.clone());
        Ok(())
    }

    pub fn node_id(&self) -> u64 {
        self.node_id
    }

    pub fn addr(&self) -> &str {
        &self.addr
    }

    pub async fn status(&self) -> ClusterStatus {
        let applied_count = self.state_machine.lock().unwrap().applied_count();
        let log_index = self.log_store.lock().unwrap().last_log_index();

        if let Some(ref raft) = self.raft {
            let rx = raft.metrics();
            let metrics = rx.borrow_watched();
            let leader_id = metrics.current_leader;
            let term = metrics.current_term;
            let state = format!("{:?}", metrics.state);

            let nodes: Vec<ClusterNodeInfo> = self
                .peers
                .iter()
                .map(|(id, addr)| ClusterNodeInfo {
                    node_id: *id,
                    addr: addr.clone(),
                    role: if Some(*id) == leader_id {
                        "leader".to_string()
                    } else {
                        "follower".to_string()
                    },
                    is_leader: Some(*id) == leader_id,
                })
                .collect();

            ClusterStatus {
                node_id: self.node_id,
                leader_id,
                nodes,
                term,
                log_index,
                applied_count,
                status: state,
            }
        } else {
            ClusterStatus {
                node_id: self.node_id,
                leader_id: None,
                nodes: vec![],
                term: 0,
                log_index,
                applied_count,
                status: "not_initialized".to_string(),
            }
        }
    }

    pub async fn propose(&self, req: ProposeRequest) -> Result<ProposeResponse, String> {
        let raft = self
            .raft
            .as_ref()
            .ok_or("raft not initialized".to_string())?;

        if let Some(ref validator) = self.conservation_validator {
            if !validator() {
                tracing::warn!("conservation check failed before propose, rejecting");
                return Err("conservation violation detected, propose rejected".to_string());
            }
        }

        let raft_req = Request {
            action: req.action,
            data: req.data,
        };

        let result = raft
            .client_write(raft_req)
            .await
            .map_err(|e| format!("client_write error: {}", e))?;

        let conservation_ok = self.conservation_validator.as_ref().map(|v| v()).unwrap_or(true);

        Ok(ProposeResponse {
            success: true,
            log_index: result.log_id.index,
            data: serde_json::json!({"committed": true}),
            conservation_verified: conservation_ok,
        })
    }

    pub async fn add_peer(&mut self, node_id: u64, addr: String) -> Result<(), String> {
        let raft = self
            .raft
            .as_ref()
            .ok_or("raft not initialized".to_string())?;

        let node = BasicNode::new(&addr);

        raft.add_learner(node_id, node, true)
            .await
            .map_err(|e| format!("add_learner error: {}", e))?;

        self.peers.insert(node_id, addr);
        Ok(())
    }

    pub async fn remove_peer(&mut self, node_id: u64) -> Result<(), String> {
        let raft = self
            .raft
            .as_ref()
            .ok_or("raft not initialized".to_string())?;

        let mut remaining: BTreeMap<u64, BasicNode> = BTreeMap::new();
        for (&id, addr) in &self.peers {
            if id != node_id {
                remaining.insert(id, BasicNode::new(addr));
            }
        }

        let voter_ids: std::collections::BTreeSet<u64> =
            remaining.keys().copied().collect();

        raft.change_membership(voter_ids, false)
            .await
            .map_err(|e| format!("change_membership error: {}", e))?;

        self.peers.remove(&node_id);
        Ok(())
    }

    pub fn is_leader(&self) -> bool {
        self.raft
            .as_ref()
            .map(|r| {
                let rx = r.metrics();
                let m = rx.borrow_watched();
                m.current_leader == Some(self.node_id)
            })
            .unwrap_or(false)
    }

    pub fn is_initialized(&self) -> bool {
        self.raft.is_some()
    }
}

struct DummyNetwork;

impl RaftNetworkFactory<TypeName> for DummyNetwork {
    type Network = DummyConn;

    async fn new_client(&mut self, _target: NodeIdOf<TypeName>, _node: &BasicNode) -> Self::Network {
        DummyConn
    }
}

struct DummyConn;

fn unreachable_err() -> openraft::error::RPCError<TypeName> {
    openraft::error::RPCError::Unreachable(openraft::error::Unreachable::new(&std::io::Error::new(
        std::io::ErrorKind::NotConnected,
        "single-node: no remote",
    )))
}

impl NetBackoff<TypeName> for DummyConn {
    fn backoff(&self) -> Backoff {
        Backoff::new(std::iter::repeat(std::time::Duration::from_secs(1)))
    }
}

impl NetVote<TypeName> for DummyConn {
    async fn vote(
        &mut self,
        _rpc: openraft::raft::VoteRequest<TypeName>,
        _option: RPCOption,
    ) -> Result<openraft::raft::VoteResponse<TypeName>, openraft::error::RPCError<TypeName>> {
        Err(unreachable_err())
    }
}

impl NetStreamAppend<TypeName> for DummyConn {
    fn stream_append<'s, S>(
        &'s mut self,
        _input: S,
        _option: RPCOption,
    ) -> openraft::base::BoxFuture<
        's,
        Result<
            openraft::base::BoxStream<
                's,
                Result<openraft::raft::StreamAppendResult<TypeName>, openraft::error::RPCError<TypeName>>,
            >,
            openraft::error::RPCError<TypeName>,
        >,
    >
    where
        S: Stream<Item = openraft::raft::AppendEntriesRequest<TypeName>> + OptionalSend + Unpin + 'static,
    {
        Box::pin(async move { Err(unreachable_err()) })
    }
}

impl NetSnapshot<TypeName> for DummyConn {
    async fn full_snapshot(
        &mut self,
        _vote: VoteOf<TypeName>,
        _snapshot: SnapshotOf<TypeName>,
        _cancel: impl Future<Output = openraft::error::ReplicationClosed> + OptionalSend + 'static,
        _option: RPCOption,
    ) -> Result<openraft::raft::SnapshotResponse<TypeName>, openraft::error::StreamingError<TypeName>> {
        Err(openraft::error::StreamingError::Unreachable(openraft::error::Unreachable::new(
            &std::io::Error::new(std::io::ErrorKind::NotConnected, "single-node: no remote"),
        )))
    }
}

impl NetTransferLeader<TypeName> for DummyConn {
    async fn transfer_leader(
        &mut self,
        _req: openraft::raft::TransferLeaderRequest<TypeName>,
        _option: RPCOption,
    ) -> Result<(), openraft::error::RPCError<TypeName>> {
        Err(unreachable_err())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cluster_manager_default() {
        let cm = ClusterManager::new(1, "127.0.0.1:3456".to_string());
        assert_eq!(cm.node_id(), 1);
        assert_eq!(cm.addr(), "127.0.0.1:3456");
        assert!(!cm.is_initialized());
        assert!(!cm.is_leader());
    }

    #[tokio::test]
    async fn single_node_init() {
        let mut cm = ClusterManager::new(1, "127.0.0.1:3457".to_string());
        cm.init_single_node().await.unwrap();
        assert!(cm.is_initialized());
        assert!(cm.is_leader());

        let status = cm.status().await;
        assert_eq!(status.node_id, 1);
        assert_eq!(status.leader_id, Some(1));
        assert_eq!(status.nodes.len(), 1);
        assert_eq!(status.nodes[0].role, "leader");
    }

    #[tokio::test]
    async fn propose_command() {
        let mut cm = ClusterManager::new(1, "127.0.0.1:3458".to_string());
        cm.init_single_node().await.unwrap();

        let resp = cm
            .propose(ProposeRequest {
                action: "encode".to_string(),
                data: serde_json::json!({"anchor": [0, 0, 0]}),
            })
            .await
            .unwrap();
        assert!(resp.success);
        assert!(resp.log_index > 0);
        assert!(resp.conservation_verified);

        let status = cm.status().await;
        assert!(status.log_index > 0);
    }

    #[test]
    fn cluster_status_serializable() {
        let status = ClusterStatus {
            node_id: 1,
            leader_id: Some(1),
            nodes: vec![ClusterNodeInfo {
                node_id: 1,
                addr: "127.0.0.1:3456".to_string(),
                role: "leader".to_string(),
                is_leader: true,
            }],
            term: 1,
            log_index: 5,
            applied_count: 3,
            status: "Leader".to_string(),
        };
        let json = serde_json::to_string(&status).unwrap();
        let parsed: ClusterStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.node_id, 1);
        assert_eq!(parsed.nodes.len(), 1);
    }
}
