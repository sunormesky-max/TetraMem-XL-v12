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
pub struct H6PhaseTransitionProposal {
    pub proposer_node: u64,
    pub super_candidates: usize,
    pub avg_edge_weight: f64,
    pub energy_budget: f64,
    pub energy_sufficient: bool,
}

impl H6PhaseTransitionProposal {
    pub fn to_propose_request(&self) -> ProposeRequest {
        ProposeRequest {
            action: "phase_transition".to_string(),
            data: serde_json::to_value(self).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyQuorumEntry {
    pub node_id: u64,
    pub available_energy: f64,
    pub conservation_ok: bool,
    pub node_count: usize,
    pub energy_sufficient: bool,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H6EnergyQuorum {
    pub quorum_id: u64,
    pub proposer: u64,
    pub required_energy_budget: f64,
    pub entries: Vec<EnergyQuorumEntry>,
    pub total_nodes: usize,
    pub quorum_threshold: usize,
    pub phase: QuorumPhase,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuorumPhase {
    Collecting,
    QuorumReached,
    QuorumFailed,
    Executed,
}

impl H6EnergyQuorum {
    pub fn new(quorum_id: u64, proposer: u64, total_nodes: usize, required_budget: f64) -> Self {
        let threshold = (total_nodes / 2) + 1;
        Self {
            quorum_id,
            proposer,
            required_energy_budget: required_budget,
            entries: Vec::new(),
            total_nodes,
            quorum_threshold: threshold,
            phase: QuorumPhase::Collecting,
        }
    }

    pub fn add_confirmation(&mut self, entry: EnergyQuorumEntry) {
        if self.phase != QuorumPhase::Collecting {
            return;
        }
        if self.entries.iter().any(|e| e.node_id == entry.node_id) {
            return;
        }
        self.entries.push(entry);
        if self.entries.len() >= self.quorum_threshold {
            if self.quorum_satisfied() {
                self.phase = QuorumPhase::QuorumReached;
            } else if self.entries.len() == self.total_nodes {
                self.phase = QuorumPhase::QuorumFailed;
            }
        }
    }

    pub fn quorum_satisfied(&self) -> bool {
        let sufficient_count = self.entries.iter().filter(|e| e.energy_sufficient).count();
        let all_conserved = self.entries.iter().all(|e| e.conservation_ok);
        sufficient_count >= self.quorum_threshold && all_conserved
    }

    pub fn total_available_energy(&self) -> f64 {
        self.entries.iter().map(|e| e.available_energy).sum()
    }

    pub fn confirming_count(&self) -> usize {
        self.entries.len()
    }

    pub fn is_reached(&self) -> bool {
        self.phase == QuorumPhase::QuorumReached
    }

    pub fn mark_executed(&mut self) {
        self.phase = QuorumPhase::Executed;
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct QuorumStatus {
    pub quorum_id: u64,
    pub phase: QuorumPhase,
    pub confirming_count: usize,
    pub total_nodes: usize,
    pub quorum_threshold: usize,
    pub total_available_energy: f64,
    pub all_conserved: bool,
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
pub type EnergyReporter = Box<dyn Fn() -> (f64, usize, bool) + Send + Sync>;

pub struct ClusterManager {
    node_id: u64,
    raft: Option<RaftNode>,
    log_store: LogStore,
    state_machine: StateMachineStore,
    addr: String,
    peers: BTreeMap<u64, String>,
    conservation_validator: Option<ConservationValidator>,
    energy_reporter: Option<EnergyReporter>,
    active_quorum: Option<H6EnergyQuorum>,
    quorum_counter: u64,
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
            energy_reporter: None,
            active_quorum: None,
            quorum_counter: 0,
        }
    }

    pub fn set_conservation_validator(&mut self, v: ConservationValidator) {
        self.conservation_validator = Some(v);
    }

    pub fn set_energy_reporter(&mut self, r: EnergyReporter) {
        self.energy_reporter = Some(r);
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

    pub fn start_energy_quorum(&mut self, required_budget: f64) -> QuorumStatus {
        self.quorum_counter += 1;
        let total_nodes = self.peers.len().max(1);
        let mut quorum = H6EnergyQuorum::new(
            self.quorum_counter,
            self.node_id,
            total_nodes,
            required_budget,
        );

        if let Some(ref reporter) = self.energy_reporter {
            let (available, node_count, conserved) = reporter();
            let entry = EnergyQuorumEntry {
                node_id: self.node_id,
                available_energy: available,
                conservation_ok: conserved,
                node_count,
                energy_sufficient: available >= required_budget,
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
            };
            quorum.add_confirmation(entry);
        }

        let status = quorum_status(&quorum);
        self.active_quorum = Some(quorum);
        status
    }

    pub fn confirm_energy_quorum(&mut self, entry: EnergyQuorumEntry) -> QuorumStatus {
        if let Some(ref mut quorum) = self.active_quorum {
            quorum.add_confirmation(entry);
            quorum_status(quorum)
        } else {
            QuorumStatus {
                quorum_id: 0,
                phase: QuorumPhase::QuorumFailed,
                confirming_count: 0,
                total_nodes: 0,
                quorum_threshold: 0,
                total_available_energy: 0.0,
                all_conserved: false,
            }
        }
    }

    pub fn get_quorum_status(&self) -> Option<QuorumStatus> {
        self.active_quorum.as_ref().map(quorum_status)
    }

    pub fn execute_quorum_transition(
        &mut self,
        _proposal: H6PhaseTransitionProposal,
    ) -> Result<ProposeResponse, String> {
        let mut quorum = self.active_quorum.take().ok_or("no active quorum")?;

        if !quorum.is_reached() {
            self.active_quorum = Some(quorum);
            return Err("quorum not reached, cannot execute phase transition".to_string());
        }

        quorum.mark_executed();
        self.active_quorum = None;
        Err("use quorum_propose() with the proposal directly after quorum check".to_string())
    }

    pub async fn quorum_propose(
        &mut self,
        proposal: H6PhaseTransitionProposal,
    ) -> Result<ProposeResponse, String> {
        match self.active_quorum {
            Some(ref q) if q.is_reached() => {
                tracing::info!(
                    quorum_id = q.quorum_id,
                    confirmations = q.confirming_count(),
                    total_energy = q.total_available_energy(),
                    "H6 energy quorum reached, executing phase transition"
                );
                let resp = self.propose(proposal.to_propose_request()).await?;
                if let Some(ref mut q) = self.active_quorum {
                    q.mark_executed();
                }
                self.active_quorum = None;
                Ok(resp)
            }
            Some(ref q) => {
                Err(format!(
                    "quorum not reached: {}/{} confirmations",
                    q.confirming_count(),
                    q.quorum_threshold
                ))
            }
            None => {
                Err("no active quorum, call start_energy_quorum first".to_string())
            }
        }
    }
}

fn quorum_status(q: &H6EnergyQuorum) -> QuorumStatus {
    QuorumStatus {
        quorum_id: q.quorum_id,
        phase: q.phase.clone(),
        confirming_count: q.confirming_count(),
        total_nodes: q.total_nodes,
        quorum_threshold: q.quorum_threshold,
        total_available_energy: q.total_available_energy(),
        all_conserved: q.entries.iter().all(|e| e.conservation_ok),
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

    #[test]
    fn energy_quorum_single_node_auto_reaches() {
        let mut q = H6EnergyQuorum::new(1, 0, 1, 100.0);
        assert_eq!(q.quorum_threshold, 1);
        assert_eq!(q.phase, QuorumPhase::Collecting);

        q.add_confirmation(EnergyQuorumEntry {
            node_id: 0,
            available_energy: 500.0,
            conservation_ok: true,
            node_count: 10,
            energy_sufficient: true,
            timestamp_ms: 1000,
        });

        assert_eq!(q.phase, QuorumPhase::QuorumReached);
        assert!(q.quorum_satisfied());
        assert!(q.is_reached());
    }

    #[test]
    fn energy_quorum_three_nodes_need_two() {
        let mut q = H6EnergyQuorum::new(1, 0, 3, 100.0);
        assert_eq!(q.quorum_threshold, 2);

        q.add_confirmation(EnergyQuorumEntry {
            node_id: 0,
            available_energy: 500.0,
            conservation_ok: true,
            node_count: 10,
            energy_sufficient: true,
            timestamp_ms: 1000,
        });
        assert_eq!(q.phase, QuorumPhase::Collecting);

        q.add_confirmation(EnergyQuorumEntry {
            node_id: 1,
            available_energy: 300.0,
            conservation_ok: true,
            node_count: 8,
            energy_sufficient: true,
            timestamp_ms: 1001,
        });
        assert_eq!(q.phase, QuorumPhase::QuorumReached);
        assert!(q.quorum_satisfied());
        assert!((q.total_available_energy() - 800.0).abs() < 1e-10);
    }

    #[test]
    fn energy_quorum_fails_if_not_conserved() {
        let mut q = H6EnergyQuorum::new(1, 0, 3, 100.0);

        q.add_confirmation(EnergyQuorumEntry {
            node_id: 0,
            available_energy: 500.0,
            conservation_ok: true,
            node_count: 10,
            energy_sufficient: true,
            timestamp_ms: 1000,
        });

        q.add_confirmation(EnergyQuorumEntry {
            node_id: 1,
            available_energy: 300.0,
            conservation_ok: false,
            node_count: 8,
            energy_sufficient: true,
            timestamp_ms: 1001,
        });

        assert!(!q.quorum_satisfied());
    }

    #[test]
    fn energy_quorum_duplicate_ignored() {
        let mut q = H6EnergyQuorum::new(1, 0, 3, 100.0);

        q.add_confirmation(EnergyQuorumEntry {
            node_id: 0,
            available_energy: 500.0,
            conservation_ok: true,
            node_count: 10,
            energy_sufficient: true,
            timestamp_ms: 1000,
        });
        q.add_confirmation(EnergyQuorumEntry {
            node_id: 0,
            available_energy: 999.0,
            conservation_ok: true,
            node_count: 20,
            energy_sufficient: true,
            timestamp_ms: 1001,
        });

        assert_eq!(q.confirming_count(), 1);
        assert!((q.total_available_energy() - 500.0).abs() < 1e-10);
    }

    #[test]
    fn energy_quorum_mark_executed() {
        let mut q = H6EnergyQuorum::new(1, 0, 1, 100.0);
        q.add_confirmation(EnergyQuorumEntry {
            node_id: 0,
            available_energy: 500.0,
            conservation_ok: true,
            node_count: 10,
            energy_sufficient: true,
            timestamp_ms: 1000,
        });
        assert_eq!(q.phase, QuorumPhase::QuorumReached);
        q.mark_executed();
        assert_eq!(q.phase, QuorumPhase::Executed);
    }

    #[tokio::test]
    async fn cluster_quorum_start_with_reporter() {
        let mut cm = ClusterManager::new(1, "127.0.0.1:3460".to_string());
        cm.init_single_node().await.unwrap();
        cm.set_energy_reporter(Box::new(|| (1000.0, 50, true)));

        let status = cm.start_energy_quorum(100.0);
        assert_eq!(status.quorum_id, 1);
        assert_eq!(status.confirming_count, 1);
        assert!(status.all_conserved);
        assert_eq!(status.phase, QuorumPhase::QuorumReached);
    }

    #[tokio::test]
    async fn cluster_quorum_propose_after_reach() {
        let mut cm = ClusterManager::new(1, "127.0.0.1:3461".to_string());
        cm.init_single_node().await.unwrap();
        cm.set_energy_reporter(Box::new(|| (1000.0, 50, true)));

        cm.start_energy_quorum(100.0);

        let proposal = H6PhaseTransitionProposal {
            proposer_node: 1,
            super_candidates: 5,
            avg_edge_weight: 3.0,
            energy_budget: 1000.0,
            energy_sufficient: true,
        };

        let resp = cm.quorum_propose(proposal).await.unwrap();
        assert!(resp.success);
        assert!(resp.log_index > 0);
    }

    #[tokio::test]
    async fn cluster_quorum_reject_without_start() {
        let mut cm = ClusterManager::new(1, "127.0.0.1:3462".to_string());
        cm.init_single_node().await.unwrap();

        let proposal = H6PhaseTransitionProposal {
            proposer_node: 1,
            super_candidates: 5,
            avg_edge_weight: 3.0,
            energy_budget: 1000.0,
            energy_sufficient: true,
        };

        let result = cm.quorum_propose(proposal).await;
        assert!(result.is_err());
    }
}
