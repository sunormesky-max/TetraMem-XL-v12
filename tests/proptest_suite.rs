use proptest::prelude::*;

use tetramem_v12::universe::coord::Coord7D;
use tetramem_v12::universe::crystal::CrystalEngine;
use tetramem_v12::universe::energy::EnergyField;
use tetramem_v12::universe::hebbian::HebbianMemory;
use tetramem_v12::universe::memory::MemoryCodec;
use tetramem_v12::universe::node::DarkUniverse;
use tetramem_v12::universe::storage::persist::{PersistEngine, UniverseSnapshot};

prop_compose! {
    fn valid_coord()(x in -50i32..50, y in -50i32..50, z in -50i32..50) -> Coord7D {
        Coord7D::new_even([x, y, z, 0, 0, 0, 0])
    }
}

proptest! {
    #[test]
    fn energy_field_preserves_total(dims in prop::array::uniform7(0.0f64..100.0)) {
        let total: f64 = dims.iter().sum();
        let field = EnergyField::from_dims(dims).unwrap();
        prop_assert!((field.total() - total).abs() < 1e-10);
    }

    #[test]
    fn energy_split_preserves_total(amount in 0.1f64..100.0, ratio in 0.0f64..1.0) {
        let mut field = EnergyField::from_dims([amount * 10.0; 7]).unwrap();
        let original = field.total();
        let split = field.split_ratio(ratio).unwrap();
        prop_assert!((split.total() + field.total() - original).abs() < 1e-10);
    }

    #[test]
    fn encode_decode_roundtrip(data in prop::collection::vec(any::<f64>(), 1..28)) {
        let mut u = DarkUniverse::new(100_000_000.0);
        let anchor = Coord7D::new_even([data.len() as i32 * 3, 10, 10, 0, 0, 0, 0]);
        let mem = match MemoryCodec::encode(&mut u, &anchor, &data) {
            Ok(m) => m,
            Err(_) => return Ok(()),
        };
        let decoded = MemoryCodec::decode(&u, &mem).unwrap();
        for (a, b) in data.iter().zip(decoded.iter()) {
            prop_assert!((a - b).abs() < 1e-10);
        }
    }

    #[test]
    fn conservation_after_materialize(coords in prop::collection::vec(valid_coord(), 0..50)) {
        let mut u = DarkUniverse::new(1_000_000.0);
        let initial_total = u.total_energy();
        for c in &coords {
            u.materialize_biased(*c, 100.0, 0.6).ok();
        }
        prop_assert!(u.verify_conservation());
        prop_assert!((u.total_energy() - initial_total).abs() < 1e-10);
    }

    #[test]
    fn transfer_preserves_conservation(
        c1 in valid_coord(),
        c2 in valid_coord(),
        amount in 1.0f64..50.0
    ) {
        let mut u = DarkUniverse::new(1_000_000.0);
        u.materialize_biased(c1, 200.0, 0.6).unwrap();
        u.materialize_biased(c2, 200.0, 0.6).unwrap();
        let _ = u.transfer_energy(&c1, &c2, amount);
        prop_assert!(u.verify_conservation());
    }

    #[test]
    fn dark_flow_preserves_conservation(
        coord in valid_coord(),
        amount in 1.0f64..50.0
    ) {
        let mut u = DarkUniverse::new(1_000_000.0);
        u.materialize_biased(coord, 200.0, 0.8).unwrap();
        u.flow_node_physical_to_dark(&coord, amount).unwrap();
        prop_assert!(u.verify_conservation());
    }

    #[test]
    fn json_roundtrip_preserves_energy(
        coords in prop::collection::vec(valid_coord(), 1..10),
        amounts in prop::collection::vec(50.0f64..200.0, 1..10)
    ) {
        let mut u = DarkUniverse::new(1_000_000.0);
        let n = coords.len().min(amounts.len());
        for i in 0..n {
            let _ = u.materialize_biased(coords[i], amounts[i], 0.6);
        }
        prop_assert!(u.verify_conservation());
        let original_total = u.total_energy();
        let original_allocated = u.allocated_energy();

        let h = HebbianMemory::new();
        let crystal = CrystalEngine::new();
        let (snapshot, _) = PersistEngine::serialize(&u, &h, &[], &crystal).unwrap();

        let json_str = serde_json::to_string(&snapshot).unwrap();
        let parsed: UniverseSnapshot =
            serde_json::from_str(&json_str).unwrap();

        let (u2, _, _, _) = PersistEngine::deserialize_relaxed(&parsed).unwrap();
        prop_assert!(u2.verify_conservation());
        prop_assert!((u2.total_energy() - original_total).abs() < 1e-6);
        prop_assert!((u2.allocated_energy() - original_allocated).abs() < 1e-6);
    }

    #[test]
    fn json_roundtrip_preserves_nodes(
        coords in prop::collection::vec(valid_coord(), 1..5),
        amounts in prop::collection::vec(50.0f64..200.0, 1..5)
    ) {
        let mut u = DarkUniverse::new(1_000_000.0);
        let n = coords.len().min(amounts.len());
        for i in 0..n {
            let _ = u.materialize_biased(coords[i], amounts[i], 0.7);
        }
        let original_count = u.active_node_count();

        let h = HebbianMemory::new();
        let crystal = CrystalEngine::new();
        let (snapshot, _) = PersistEngine::serialize(&u, &h, &[], &crystal).unwrap();

        let json_str = serde_json::to_string(&snapshot).unwrap();
        let parsed: UniverseSnapshot =
            serde_json::from_str(&json_str).unwrap();

        let (u2, _, _, _) = PersistEngine::deserialize_relaxed(&parsed).unwrap();
        prop_assert_eq!(u2.active_node_count(), original_count);
        prop_assert!(u2.verify_conservation());
    }

    #[test]
    fn json_roundtrip_preserves_hebbian(
        edges in prop::collection::vec(
            (valid_coord(), valid_coord(), 0.1f64..5.0),
            1..10
        )
    ) {
        let mut u = DarkUniverse::new(1_000_000.0);
        let mut h = HebbianMemory::new();
        for (i, (a, b, w)) in edges.iter().enumerate() {
            let _ = u.materialize_biased(*a, 100.0, 0.6);
            let _ = u.materialize_biased(*b, 100.0, 0.6);
            if *a != *b {
                h.boost_edge(a, b, *w);
            }
            let _ = i;
        }
        let original_edge_count = h.edge_count();
        let original_total_weight: f64 = h.total_weight();

        let crystal = CrystalEngine::new();
        let (snapshot, _) = PersistEngine::serialize(&u, &h, &[], &crystal).unwrap();

        let json_str = serde_json::to_string(&snapshot).unwrap();
        let parsed: UniverseSnapshot =
            serde_json::from_str(&json_str).unwrap();

        let (_, h2, _, _) = PersistEngine::deserialize_relaxed(&parsed).unwrap();
        prop_assert_eq!(h2.edge_count(), original_edge_count);
        prop_assert!((h2.total_weight() - original_total_weight).abs() < 1e-6);
    }

    #[test]
    fn encode_serialize_roundtrip_preserves_data(
        data in prop::collection::vec(any::<f64>(), 1..7)
    ) {
        let mut u = DarkUniverse::new(1_000_000.0);
        let anchor = Coord7D::new_even([data.len() as i32 * 5, 5, 5, 0, 0, 0, 0]);
        let mem = match MemoryCodec::encode(&mut u, &anchor, &data) {
            Ok(m) => m,
            Err(_) => return Ok(()),
        };

        let h = HebbianMemory::new();
        let crystal = CrystalEngine::new();
        let (snapshot, _) = PersistEngine::serialize(&u, &h, std::slice::from_ref(&mem), &crystal).unwrap();

        let json_str = serde_json::to_string(&snapshot).unwrap();
        let parsed: UniverseSnapshot =
            serde_json::from_str(&json_str).unwrap();

        let (u2, _, mems2, _) = PersistEngine::deserialize_relaxed(&parsed).unwrap();
        prop_assert!(u2.verify_conservation());
        prop_assert_eq!(mems2.len(), 1);
        let decoded = MemoryCodec::decode(&u2, &mems2[0]).unwrap();
        for (a, b) in data.iter().zip(decoded.iter()) {
            prop_assert!((a - b).abs() < 1e-8);
        }
    }
}
