use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};

#[derive(Debug, Clone)]
pub enum UniverseEvent {
    MemoryEncoded {
        anchor: [i32; 3],
        data_dim: usize,
        importance: f64,
    },
    MemoryDecoded {
        anchor: [i32; 3],
    },
    PulseCompleted {
        source: [i32; 7],
        pulse_type: String,
        visited_nodes: usize,
        paths_recorded: usize,
    },
    CrystalFormed {
        new_crystals: usize,
        new_super: usize,
        total_crystals: usize,
    },
    DreamCompleted {
        phase: String,
        paths_replayed: usize,
        paths_weakened: usize,
        memories_consolidated: usize,
        memories_merged: usize,
        edges_before: usize,
        edges_after: usize,
    },
    PhaseTransition {
        super_candidates: usize,
        phase_coherent: bool,
        requires_consensus: bool,
    },
    RegulationCycle {
        stress_level: f64,
        entropy: f64,
        actions_count: usize,
    },
    ScaleEvent {
        nodes_added: usize,
        energy_expanded_by: f64,
        reason: String,
    },
    ConservationViolation {
        drift: f64,
        active_nodes: usize,
    },
    BackupCreated {
        backup_id: u64,
        bytes: usize,
        conservation_ok: bool,
    },
}

impl std::fmt::Display for UniverseEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MemoryEncoded {
                anchor,
                data_dim,
                importance,
            } => {
                write!(
                    f,
                    "MemoryEncoded(anchor={:?} dim={} imp={:.2})",
                    anchor, data_dim, importance
                )
            }
            Self::MemoryDecoded { anchor } => {
                write!(f, "MemoryDecoded(anchor={:?})", anchor)
            }
            Self::PulseCompleted {
                source,
                pulse_type,
                visited_nodes,
                paths_recorded,
            } => {
                write!(
                    f,
                    "PulseCompleted(src={:?} type={} visited={} paths={})",
                    source, pulse_type, visited_nodes, paths_recorded
                )
            }
            Self::CrystalFormed {
                new_crystals,
                new_super,
                total_crystals,
            } => {
                write!(
                    f,
                    "CrystalFormed(new={} super={} total={})",
                    new_crystals, new_super, total_crystals
                )
            }
            Self::DreamCompleted {
                phase,
                paths_replayed,
                paths_weakened,
                memories_consolidated,
                memories_merged,
                edges_before,
                edges_after,
            } => {
                write!(f, "DreamCompleted(phase={} replay={} weaken={} consolidate={} merge={} edges:{}→{})", 
                    phase, paths_replayed, paths_weakened, memories_consolidated, memories_merged, edges_before, edges_after)
            }
            Self::PhaseTransition {
                super_candidates,
                phase_coherent,
                requires_consensus,
            } => {
                write!(
                    f,
                    "PhaseTransition(candidates={} coherent={} consensus={})",
                    super_candidates, phase_coherent, requires_consensus
                )
            }
            Self::RegulationCycle {
                stress_level,
                entropy,
                actions_count,
            } => {
                write!(
                    f,
                    "RegulationCycle(stress={:.2} entropy={:.2} actions={})",
                    stress_level, entropy, actions_count
                )
            }
            Self::ScaleEvent {
                nodes_added,
                energy_expanded_by,
                reason,
            } => {
                write!(
                    f,
                    "ScaleEvent(+{}nodes +{:.0}E reason={})",
                    nodes_added, energy_expanded_by, reason
                )
            }
            Self::ConservationViolation {
                drift,
                active_nodes,
            } => {
                write!(
                    f,
                    "ConservationViolation(drift={:.e} nodes={})",
                    drift, active_nodes
                )
            }
            Self::BackupCreated {
                backup_id,
                bytes,
                conservation_ok,
            } => {
                write!(
                    f,
                    "BackupCreated(id={} bytes={} cons={})",
                    backup_id, bytes, conservation_ok
                )
            }
        }
    }
}

pub type SubscriberId = u64;

type BoxedHandler = Box<dyn Fn(&UniverseEvent) + Send + Sync>;

pub struct EventBus {
    sender: Sender<UniverseEvent>,
    receiver: Receiver<UniverseEvent>,
    subscribers: HashMap<SubscriberId, BoxedHandler>,
    next_sub_id: SubscriberId,
    history: Vec<UniverseEvent>,
    max_history: usize,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            sender: tx,
            receiver: rx,
            subscribers: HashMap::new(),
            next_sub_id: 1,
            history: Vec::new(),
            max_history: 1000,
        }
    }

    pub fn with_history_capacity(mut self, cap: usize) -> Self {
        self.max_history = cap;
        self
    }

    pub fn publish(&self, event: UniverseEvent) {
        if self.sender.send(event).is_err() {
            tracing::warn!("eventbus publish: receiver dropped, event discarded");
        }
    }

    pub fn subscribe(
        &mut self,
        handler: impl Fn(&UniverseEvent) + Send + Sync + 'static,
    ) -> SubscriberId {
        let id = self.next_sub_id;
        self.next_sub_id += 1;
        self.subscribers.insert(id, Box::new(handler));
        id
    }

    pub fn unsubscribe(&mut self, id: SubscriberId) -> bool {
        self.subscribers.remove(&id).is_some()
    }

    pub fn drain(&mut self) -> usize {
        let mut count = 0;
        loop {
            match self.receiver.try_recv() {
                Ok(event) => {
                    for handler in self.subscribers.values() {
                        handler(&event);
                    }
                    if self.history.len() >= self.max_history {
                        self.history.remove(0);
                    }
                    self.history.push(event);
                    count += 1;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        count
    }

    pub fn sender(&self) -> EventBusSender {
        EventBusSender {
            inner: self.sender.clone(),
        }
    }

    pub fn history(&self) -> &[UniverseEvent] {
        &self.history
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

#[derive(Clone)]
pub struct EventBusSender {
    inner: Sender<UniverseEvent>,
}

impl EventBusSender {
    pub fn publish(&self, event: UniverseEvent) {
        if self.inner.send(event).is_err() {
            tracing::warn!("EventBusSender publish: receiver dropped, event discarded");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn event_bus_pub_sub() {
        let mut bus = EventBus::new();
        let received = Arc::new(Mutex::new(Vec::new()));
        let recv_clone = received.clone();
        bus.subscribe(move |evt| {
            recv_clone.lock().unwrap().push(format!("{}", evt));
        });

        bus.publish(UniverseEvent::MemoryEncoded {
            anchor: [1, 2, 3],
            data_dim: 7,
            importance: 0.8,
        });
        bus.publish(UniverseEvent::PulseCompleted {
            source: [1, 2, 3, 0, 0, 0, 0],
            pulse_type: "exploratory".into(),
            visited_nodes: 42,
            paths_recorded: 5,
        });

        let drained = bus.drain();
        assert_eq!(drained, 2);
        assert_eq!(bus.history_len(), 2);
        let msgs = received.lock().unwrap();
        assert_eq!(msgs.len(), 2);
        assert!(msgs[0].contains("MemoryEncoded"));
        assert!(msgs[1].contains("PulseCompleted"));
    }

    #[test]
    fn event_bus_unsubscribe() {
        let mut bus = EventBus::new();
        let id = bus.subscribe(|_| {});
        assert_eq!(bus.subscriber_count(), 1);
        assert!(bus.unsubscribe(id));
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[test]
    fn event_bus_sender_clone() {
        let mut bus = EventBus::new();
        let sender = bus.sender();
        let received = Arc::new(Mutex::new(Vec::new()));
        let recv_clone = received.clone();
        bus.subscribe(move |evt| {
            recv_clone.lock().unwrap().push(format!("{}", evt));
        });

        sender.publish(UniverseEvent::CrystalFormed {
            new_crystals: 3,
            new_super: 1,
            total_crystals: 10,
        });
        bus.drain();
        assert_eq!(bus.history_len(), 1);
    }

    #[test]
    fn event_bus_history_cap() {
        let mut bus = EventBus::new().with_history_capacity(3);
        bus.subscribe(|_| {});
        for i in 0..5u32 {
            bus.publish(UniverseEvent::BackupCreated {
                backup_id: i as u64,
                bytes: 100,
                conservation_ok: true,
            });
            bus.drain();
        }
        assert_eq!(bus.history_len(), 3);
    }

    #[test]
    fn event_display_format() {
        let evt = UniverseEvent::DreamCompleted {
            phase: "Consolidate".into(),
            paths_replayed: 10,
            paths_weakened: 3,
            memories_consolidated: 5,
            memories_merged: 2,
            edges_before: 20,
            edges_after: 18,
        };
        let s = format!("{}", evt);
        assert!(s.contains("DreamCompleted"));
        assert!(s.contains("replay=10"));
        assert!(s.contains("merge=2"));
    }
}
