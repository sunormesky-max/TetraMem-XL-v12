// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use std::collections::{HashMap, VecDeque};

const CHANNEL_CAPACITY: usize = 4096;

#[derive(Debug, Clone)]
pub enum UniverseEvent {
    MemoryEncoded {
        anchor: [i32; 7],
        data_dim: usize,
        importance: f64,
    },
    MemoryDecoded {
        anchor: [i32; 7],
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
    receiver: std::sync::mpsc::Receiver<UniverseEvent>,
    subscribers: HashMap<SubscriberId, BoxedHandler>,
    next_sub_id: SubscriberId,
    history: VecDeque<UniverseEvent>,
    max_history: usize,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::sync_channel(CHANNEL_CAPACITY);
        std::mem::forget(tx);
        Self {
            receiver: rx,
            subscribers: HashMap::new(),
            next_sub_id: 1,
            history: VecDeque::new(),
            max_history: 1000,
        }
    }

    pub fn with_history_capacity(mut self, cap: usize) -> Self {
        self.max_history = cap;
        self
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
                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            handler(&event);
                        }));
                    }
                    if self.history.len() >= self.max_history {
                        self.history.pop_front();
                    }
                    self.history.push_back(event);
                    count += 1;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
            }
        }
        count
    }

    pub fn sender(&self) -> EventBusSender {
        let (tx, _) = std::sync::mpsc::sync_channel(CHANNEL_CAPACITY);
        EventBusSender { inner: tx }
    }

    pub fn create_channel() -> (EventBusSender, std::sync::mpsc::Receiver<UniverseEvent>) {
        let (tx, rx) = std::sync::mpsc::sync_channel(CHANNEL_CAPACITY);
        (EventBusSender { inner: tx }, rx)
    }

    pub fn from_receiver(rx: std::sync::mpsc::Receiver<UniverseEvent>) -> Self {
        Self {
            receiver: rx,
            subscribers: HashMap::new(),
            next_sub_id: 1,
            history: VecDeque::new(),
            max_history: 1000,
        }
    }

    pub fn history(&self) -> Vec<&UniverseEvent> {
        self.history.iter().collect()
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

    pub fn pending_count(&self) -> usize {
        self.receiver.try_iter().count()
    }
}

#[derive(Clone)]
pub struct EventBusSender {
    inner: std::sync::mpsc::SyncSender<UniverseEvent>,
}

impl EventBusSender {
    pub fn publish(&self, event: UniverseEvent) {
        if self.inner.send(event).is_err() {
            tracing::warn!("EventBusSender: receiver dropped, event discarded");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn create_pair() -> (EventBusSender, EventBus) {
        let (tx, rx) = EventBus::create_channel();
        (tx, EventBus::from_receiver(rx))
    }

    #[test]
    fn event_bus_pub_sub() {
        let (sender, mut bus) = create_pair();
        let received = Arc::new(Mutex::new(Vec::new()));
        let recv_clone = received.clone();
        bus.subscribe(move |evt| {
            recv_clone.lock().unwrap().push(format!("{}", evt));
        });

        sender.publish(UniverseEvent::MemoryEncoded {
            anchor: [1, 2, 3, 0, 0, 0, 0],
            data_dim: 7,
            importance: 0.8,
        });
        sender.publish(UniverseEvent::PulseCompleted {
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
    fn sender_clone_and_publish() {
        let (sender, mut bus) = create_pair();
        bus.subscribe(|_| {});
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
        let (sender, mut bus) = create_pair();
        bus = bus.with_history_capacity(3);
        bus.subscribe(|_| {});
        for i in 0..5u32 {
            sender.publish(UniverseEvent::BackupCreated {
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

    #[test]
    fn subscriber_panic_doesnt_kill_drain() {
        let (sender, mut bus) = create_pair();
        let received = Arc::new(Mutex::new(0usize));
        let recv_clone = received.clone();
        bus.subscribe(move |_| {
            panic!("intentional test panic");
        });
        bus.subscribe(move |_: &UniverseEvent| {
            *recv_clone.lock().unwrap() += 1;
        });
        sender.publish(UniverseEvent::BackupCreated {
            backup_id: 1,
            bytes: 100,
            conservation_ok: true,
        });
        let drained = bus.drain();
        assert_eq!(drained, 1);
        assert_eq!(*received.lock().unwrap(), 1);
    }
}
