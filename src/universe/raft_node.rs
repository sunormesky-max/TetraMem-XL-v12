use std::collections::BTreeMap;
use std::fmt;
use std::fmt::Debug;
use std::io;
use std::io::Cursor;
use std::ops::Bound;
use std::ops::RangeBounds;
use std::sync::{Arc, Mutex};

use futures::Stream;
use openraft::entry::RaftEntry;
use openraft::storage::IOFlushed;
use openraft::storage::LogState;
use openraft::storage::RaftLogReader;
use openraft::storage::RaftLogStorage;
use openraft::storage::RaftSnapshotBuilder;
use openraft::storage::RaftStateMachine;
use openraft::type_config::alias::EntryOf;
use openraft::type_config::alias::LogIdOf;
use openraft::type_config::alias::SnapshotDataOf;
use openraft::type_config::alias::SnapshotMetaOf;
use openraft::type_config::alias::SnapshotOf;
use openraft::type_config::alias::StoredMembershipOf;
use openraft::type_config::alias::VoteOf;
use openraft::EntryPayload;
use openraft::OptionalSend;
use openraft::TokioRuntime;
use serde::{Deserialize, Serialize};

pub type NodeId = u64;

openraft::declare_raft_types!(
    pub TypeName:
        D = Request,
        R = Response,
        AsyncRuntime = TokioRuntime,
);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub action: String,
    pub data: serde_json::Value,
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Request({})", self.action)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    pub data: serde_json::Value,
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Response(success={})", self.success)
    }
}

pub type LogStore = Arc<Mutex<LogStoreInner>>;
pub type StateMachineStore = Arc<Mutex<StateMachineInner>>;

#[derive(Debug)]
pub struct LogStoreInner {
    last_purged: Option<LogIdOf<TypeName>>,
    log: BTreeMap<u64, EntryOf<TypeName>>,
    vote: Option<VoteOf<TypeName>>,
}

impl Default for LogStoreInner {
    fn default() -> Self {
        Self {
            last_purged: None,
            log: BTreeMap::new(),
            vote: None,
        }
    }
}

impl LogStoreInner {
    pub fn last_log_index(&self) -> u64 {
        self.log.iter().next_back().map(|(_, e)| e.log_id().index).unwrap_or(0)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StateMachineInner {
    last_applied: Option<LogIdOf<TypeName>>,
    #[serde(skip)]
    snapshot: Option<Vec<u8>>,
    applied_commands: Vec<(String, serde_json::Value)>,
}

impl StateMachineInner {
    pub fn applied_count(&self) -> usize {
        self.applied_commands.len()
    }
}

pub fn new_log_store() -> LogStore {
    Arc::new(Mutex::new(LogStoreInner::default()))
}

pub fn new_state_machine() -> StateMachineStore {
    Arc::new(Mutex::new(StateMachineInner::default()))
}

fn snap_meta(sm: &StateMachineInner) -> SnapshotMetaOf<TypeName> {
    SnapshotMetaOf::<TypeName> {
        last_log_id: sm.last_applied,
        last_membership: StoredMembershipOf::<TypeName>::default(),
        snapshot_id: format!("snap-{}", sm.applied_count()),
    }
}

impl RaftLogReader<TypeName> for LogStore {
    async fn try_get_log_entries<RB: RangeBounds<u64> + Clone + Debug + OptionalSend>(
        &mut self,
        range: RB,
    ) -> Result<Vec<EntryOf<TypeName>>, io::Error> {
        let inner = self.lock().unwrap();
        let start = match range.start_bound().cloned() {
            Bound::Included(s) => s,
            Bound::Excluded(s) => s + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound().cloned() {
            Bound::Included(e) => e + 1,
            Bound::Excluded(e) => e,
            Bound::Unbounded => u64::MAX,
        };
        let entries: Vec<_> = inner
            .log
            .range(start..end)
            .map(|(_, v)| v.clone())
            .collect();
        Ok(entries)
    }

    async fn read_vote(&mut self) -> Result<Option<VoteOf<TypeName>>, io::Error> {
        Ok(self.lock().unwrap().vote.clone())
    }
}

impl RaftSnapshotBuilder<TypeName> for StateMachineStore {
    async fn build_snapshot(&mut self) -> Result<SnapshotOf<TypeName>, io::Error> {
        let sm = self.lock().unwrap();
        let data = serde_json::to_vec(&*sm)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let meta = snap_meta(&sm);
        Ok(SnapshotOf::<TypeName> {
            meta,
            snapshot: Cursor::new(data),
        })
    }
}

impl RaftLogStorage<TypeName> for LogStore {
    type LogReader = LogStore;

    async fn get_log_state(&mut self) -> Result<LogState<TypeName>, io::Error> {
        let inner = self.lock().unwrap();
        let last_log_id = inner.log.iter().next_back().map(|(_, e)| e.log_id());
        Ok(LogState {
            last_purged_log_id: inner.last_purged,
            last_log_id,
        })
    }

    async fn get_log_reader(&mut self) -> Self::LogReader {
        self.clone()
    }

    async fn save_vote(&mut self, vote: &VoteOf<TypeName>) -> Result<(), io::Error> {
        self.lock().unwrap().vote = Some(vote.clone());
        Ok(())
    }

    async fn append<I>(
        &mut self,
        entries: I,
        callback: IOFlushed<TypeName>,
    ) -> Result<(), io::Error>
    where
        I: IntoIterator<Item = EntryOf<TypeName>> + OptionalSend,
        I::IntoIter: OptionalSend,
    {
        {
            let mut inner = self.lock().unwrap();
            for entry in entries {
                inner.log.insert(entry.log_id().index, entry);
            }
        }
        callback.io_completed(Ok(()));
        Ok(())
    }

    async fn truncate_after(
        &mut self,
        last_log_id: Option<LogIdOf<TypeName>>,
    ) -> Result<(), io::Error> {
        let mut inner = self.lock().unwrap();
        if let Some(ref lid) = last_log_id {
            let keys_to_remove: Vec<u64> = inner
                .log
                .range((lid.index + 1)..)
                .map(|(k, _)| *k)
                .collect();
            for k in keys_to_remove {
                inner.log.remove(&k);
            }
        } else {
            inner.log.clear();
        }
        Ok(())
    }

    async fn purge(&mut self, log_id: LogIdOf<TypeName>) -> Result<(), io::Error> {
        let mut inner = self.lock().unwrap();
        let keys_to_remove: Vec<u64> = inner
            .log
            .range(..=log_id.index)
            .map(|(k, _)| *k)
            .collect();
        for k in keys_to_remove {
            inner.log.remove(&k);
        }
        inner.last_purged = Some(log_id);
        Ok(())
    }
}

impl RaftStateMachine<TypeName> for StateMachineStore {
    type SnapshotBuilder = StateMachineStore;

    async fn applied_state(
        &mut self,
    ) -> Result<(Option<LogIdOf<TypeName>>, StoredMembershipOf<TypeName>), io::Error> {
        let sm = self.lock().unwrap();
        Ok((sm.last_applied, StoredMembershipOf::<TypeName>::default()))
    }

    async fn apply<Strm>(&mut self, entries: Strm) -> Result<(), io::Error>
    where
        Strm: Stream<
                Item = Result<openraft::storage::EntryResponder<TypeName>, io::Error>,
            > + Unpin
            + OptionalSend,
    {
        use futures::StreamExt;
        tokio::pin!(entries);
        while let Some(item) = entries.next().await {
            let (entry, responder) = item?;
            let mut sm = self.lock().unwrap();
            sm.last_applied = Some(entry.log_id());
            if let EntryPayload::Normal(ref req) = entry.payload {
                sm.applied_commands
                    .push((req.action.clone(), req.data.clone()));
            }
            drop(sm);
            if let Some(r) = responder {
                r.send(Response {
                    success: true,
                    data: serde_json::json!({"applied": true}),
                });
            }
        }
        Ok(())
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        self.clone()
    }

    async fn begin_receiving_snapshot(
        &mut self,
    ) -> Result<SnapshotDataOf<TypeName>, io::Error> {
        Ok(Cursor::new(vec![]))
    }

    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMetaOf<TypeName>,
        snapshot: SnapshotDataOf<TypeName>,
    ) -> Result<(), io::Error> {
        let mut sm = self.lock().unwrap();
        sm.last_applied = meta.last_log_id;
        sm.snapshot = Some(snapshot.into_inner());
        Ok(())
    }

    async fn get_current_snapshot(
        &mut self,
    ) -> Result<Option<SnapshotOf<TypeName>>, io::Error> {
        let sm = self.lock().unwrap();
        if sm.snapshot.is_some() || sm.last_applied.is_some() {
            let data = serde_json::to_vec(&*sm)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let meta = snap_meta(&sm);
            Ok(Some(SnapshotOf::<TypeName> {
                meta,
                snapshot: Cursor::new(data),
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_store_default_is_empty() {
        let store = LogStoreInner::default();
        assert!(store.log.is_empty());
        assert!(store.last_purged.is_none());
        assert!(store.vote.is_none());
    }

    #[test]
    fn state_machine_default_is_empty() {
        let sm = StateMachineInner::default();
        assert!(sm.last_applied.is_none());
        assert!(sm.snapshot.is_none());
        assert_eq!(sm.applied_count(), 0);
    }

    #[test]
    fn request_response_serializable() {
        let req = Request {
            action: "encode".to_string(),
            data: serde_json::json!({"anchor": [0, 0, 0]}),
        };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: Request = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.action, "encode");

        let resp = Response {
            success: true,
            data: serde_json::json!({"id": 1}),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: Response = serde_json::from_str(&json).unwrap();
        assert!(parsed.success);
    }
}
