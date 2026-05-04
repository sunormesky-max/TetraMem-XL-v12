// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
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

// NOTE: std::sync::Mutex is intentional here — openraft's RaftLogStorage/RaftStateMachine
// traits call async methods with &mut self, and the Arc<Mutex<>> wrapper is held briefly.
// tokio::Mutex would add unnecessary overhead since lock hold times are always short
// (in-memory BTreeMap + optional SQLite write). The lock_failed handler ensures
// poison is treated as an I/O error rather than a panic.
pub type LogStore = Arc<Mutex<LogStoreInner>>;
pub type StateMachineStore = Arc<Mutex<StateMachineInner>>;

fn lock_failed<T>(e: std::sync::PoisonError<T>) -> io::Error {
    io::Error::other(format!("lock poisoned: {}", e))
}

fn compute_hmac_with_key(data: &str, key: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac =
        HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    format!("{:x}", mac.finalize().into_bytes())
}

#[derive(Debug, Default)]
pub struct LogStoreInner {
    last_purged: Option<LogIdOf<TypeName>>,
    log: BTreeMap<u64, EntryOf<TypeName>>,
    vote: Option<VoteOf<TypeName>>,
    db: Option<rusqlite::Connection>,
    hmac_key: String,
}

impl LogStoreInner {
    pub fn with_hmac_key(mut self, key: &str) -> Self {
        self.hmac_key = if key.is_empty() {
            "tetramem-raft-log-v1".to_string()
        } else {
            key.to_string()
        };
        self
    }

    pub fn last_log_index(&self) -> u64 {
        self.log
            .iter()
            .next_back()
            .map(|(_, e)| e.log_id().index)
            .unwrap_or(0)
    }

    fn compute_entry_hmac(&self, data: &str) -> String {
        compute_hmac_with_key(data, &self.hmac_key)
    }

    fn persist_entry(&self, index: u64, entry: &EntryOf<TypeName>) -> Result<(), String> {
        if let Some(ref db) = self.db {
            let data = serde_json::to_string(entry)
                .map_err(|e| format!("serialize raft log entry {}: {}", index, e))?;
            let hash = self.compute_entry_hmac(&data);
            db.execute(
                "INSERT OR REPLACE INTO raft_log (idx, data, hmac) VALUES (?1, ?2, ?3)",
                rusqlite::params![index as i64, &data, &hash],
            )
            .map_err(|e| format!("persist raft log entry {}: {}", index, e))?;
        }
        Ok(())
    }

    fn persist_vote(&self) {
        if let Some(ref db) = self.db {
            if let Some(ref v) = self.vote {
                let data = match serde_json::to_string(v) {
                    Ok(d) => d,
                    Err(e) => {
                        tracing::error!("failed to serialize raft vote: {}", e);
                        return;
                    }
                };
                if let Err(e) = db.execute(
                    "INSERT OR REPLACE INTO raft_meta (key, value) VALUES ('vote', ?1)",
                    rusqlite::params![&data],
                ) {
                    tracing::error!("failed to persist raft vote: {}", e);
                }
            }
        }
    }

    fn persist_purged(&self) {
        if let Some(ref db) = self.db {
            if let Some(ref lid) = self.last_purged {
                let data = match serde_json::to_string(lid) {
                    Ok(d) => d,
                    Err(e) => {
                        tracing::error!("failed to serialize last_purged: {}", e);
                        return;
                    }
                };
                if let Err(e) = db.execute(
                    "INSERT OR REPLACE INTO raft_meta (key, value) VALUES ('last_purged', ?1)",
                    rusqlite::params![&data],
                ) {
                    tracing::error!("failed to persist last_purged: {}", e);
                }
            }
        }
    }

    fn delete_range(&self, start: u64, end: u64) {
        if let Some(ref db) = self.db {
            if let Err(e) = db.execute(
                "DELETE FROM raft_log WHERE idx >= ?1 AND idx < ?2",
                rusqlite::params![start as i64, end as i64],
            ) {
                tracing::error!("failed to delete raft log range {}..{}: {}", start, end, e);
            }
        }
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

pub fn new_log_store_with_persistence(
    db_path: &std::path::Path,
    hmac_key: &str,
) -> Result<LogStore, io::Error> {
    let conn = rusqlite::Connection::open(db_path)
        .map_err(|e| io::Error::other(format!("failed to open raft log db: {}", e)))?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS raft_log (
            idx INTEGER PRIMARY KEY,
            data TEXT NOT NULL,
            hmac TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS raft_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )
    .map_err(|e| io::Error::other(format!("raft log db init: {}", e)))?;

    let mut inner = LogStoreInner::default().with_hmac_key(hmac_key);

    {
        let mut stmt = conn
            .prepare("SELECT idx, data, hmac FROM raft_log ORDER BY idx")
            .map_err(|e| io::Error::other(format!("raft log load: {}", e)))?;
        let rows = stmt
            .query_map([], |row| {
                let idx: i64 = row.get(0)?;
                let data: String = row.get(1)?;
                let hmac: String = row.get(2).unwrap_or_default();
                Ok((idx, data, hmac))
            })
            .map_err(|e| io::Error::other(format!("raft log query: {}", e)))?;

        for row in rows {
            let (idx, data, hmac) =
                row.map_err(|e| io::Error::other(format!("raft log row: {}", e)))?;
            if !hmac.is_empty() {
                let expected = inner.compute_entry_hmac(&data);
                if hmac != expected {
                    tracing::error!(
                        "HMAC mismatch on raft log entry {} — data may be tampered",
                        idx
                    );
                    return Err(io::Error::other(format!(
                        "HMAC integrity check failed for raft log entry {}. Data may have been tampered with.",
                        idx
                    )));
                }
            }
            match serde_json::from_str::<EntryOf<TypeName>>(&data) {
                Ok(entry) => {
                    inner.log.insert(idx as u64, entry);
                }
                Err(e) => {
                    tracing::error!("corrupt raft log entry at index {}: {} — aborting load to prevent data inconsistency", idx, e);
                    return Err(io::Error::other(format!(
                        "corrupt raft log entry at index {}: {}. Remove or repair the raft log database before restarting.",
                        idx, e
                    )));
                }
            }
        }
    }

    {
        if let Ok(val) = conn.query_row(
            "SELECT value FROM raft_meta WHERE key = 'vote'",
            [],
            |row| row.get::<_, String>(0),
        ) {
            match serde_json::from_str::<VoteOf<TypeName>>(&val) {
                Ok(v) => inner.vote = Some(v),
                Err(e) => tracing::warn!("failed to deserialize raft vote: {}", e),
            }
        }
    }

    {
        if let Ok(val) = conn.query_row(
            "SELECT value FROM raft_meta WHERE key = 'last_purged'",
            [],
            |row| row.get::<_, String>(0),
        ) {
            match serde_json::from_str::<LogIdOf<TypeName>>(&val) {
                Ok(lid) => inner.last_purged = Some(lid),
                Err(e) => tracing::warn!("failed to deserialize last_purged: {}", e),
            }
        }
    }

    inner.db = Some(conn);
    tracing::info!(
        "raft log store loaded with {} entries from {}",
        inner.log.len(),
        db_path.display()
    );
    Ok(Arc::new(Mutex::new(inner)))
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
        let inner = self.lock().map_err(lock_failed)?;
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
        Ok(self.lock().map_err(lock_failed)?.vote)
    }
}

impl RaftSnapshotBuilder<TypeName> for StateMachineStore {
    async fn build_snapshot(&mut self) -> Result<SnapshotOf<TypeName>, io::Error> {
        let sm = self.lock().map_err(lock_failed)?;
        let data = serde_json::to_vec(&*sm).map_err(io::Error::other)?;
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
        let inner = self.lock().map_err(lock_failed)?;
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
        {
            let mut inner = self.lock().map_err(lock_failed)?;
            inner.vote = Some(*vote);
            inner.persist_vote();
        }
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
            let mut inner = self.lock().map_err(lock_failed)?;
            for entry in entries {
                inner
                    .persist_entry(entry.log_id().index, &entry)
                    .map_err(io::Error::other)?;
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
        let mut inner = self.lock().map_err(lock_failed)?;
        if let Some(ref lid) = last_log_id {
            let keys_to_remove: Vec<u64> = inner
                .log
                .range((lid.index + 1)..)
                .map(|(k, _)| *k)
                .collect();
            inner.delete_range(lid.index + 1, inner.last_log_index() + 1);
            for k in keys_to_remove {
                inner.log.remove(&k);
            }
        } else {
            inner.delete_range(0, inner.last_log_index() + 1);
            inner.log.clear();
        }
        Ok(())
    }

    async fn purge(&mut self, log_id: LogIdOf<TypeName>) -> Result<(), io::Error> {
        let mut inner = self.lock().map_err(lock_failed)?;
        let keys_to_remove: Vec<u64> = inner.log.range(..=log_id.index).map(|(k, _)| *k).collect();
        inner.delete_range(0, log_id.index + 1);
        for k in keys_to_remove {
            inner.log.remove(&k);
        }
        inner.last_purged = Some(log_id);
        inner.persist_purged();
        Ok(())
    }
}

// DESIGN NOTE: The Raft state machine records commands to `applied_commands` for
// audit/durability but does NOT execute them against the DarkUniverse. This is intentional:
// full state replication would require AppState access inside async Raft trait methods,
// which introduces complex lock ordering and error recovery concerns. Instead, Raft provides
// log replication (consensus on command ordering), and a separate replay/applier layer can
// consume `applied_commands` to apply state changes. See ARCHITECTURE.md for the full design.
impl RaftStateMachine<TypeName> for StateMachineStore {
    type SnapshotBuilder = StateMachineStore;

    async fn applied_state(
        &mut self,
    ) -> Result<(Option<LogIdOf<TypeName>>, StoredMembershipOf<TypeName>), io::Error> {
        let sm = self.lock().map_err(lock_failed)?;
        Ok((sm.last_applied, StoredMembershipOf::<TypeName>::default()))
    }

    async fn apply<Strm>(&mut self, entries: Strm) -> Result<(), io::Error>
    where
        Strm: Stream<Item = Result<openraft::storage::EntryResponder<TypeName>, io::Error>>
            + Unpin
            + OptionalSend,
    {
        use futures::StreamExt;
        tokio::pin!(entries);
        while let Some(item) = entries.next().await {
            let (entry, responder) = item?;
            let mut sm = self.lock().map_err(lock_failed)?;
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

    async fn begin_receiving_snapshot(&mut self) -> Result<SnapshotDataOf<TypeName>, io::Error> {
        Ok(Cursor::new(vec![]))
    }

    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMetaOf<TypeName>,
        snapshot: SnapshotDataOf<TypeName>,
    ) -> Result<(), io::Error> {
        let mut sm = self.lock().map_err(lock_failed)?;
        sm.last_applied = meta.last_log_id;
        sm.snapshot = Some(snapshot.into_inner());
        Ok(())
    }

    async fn get_current_snapshot(&mut self) -> Result<Option<SnapshotOf<TypeName>>, io::Error> {
        let sm = self.lock().map_err(lock_failed)?;
        if sm.snapshot.is_some() || sm.last_applied.is_some() {
            let data = serde_json::to_vec(&*sm).map_err(io::Error::other)?;
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
