use std::collections::HashMap;
use std::time::Instant;

use tetramem_v12::universe::*;

const SEPARATOR: &str = "══════════════════════════════════════════════════════════════════════";
const HEADER: &str = "╔════════════════════════════════════════════════════════════════╗";
const FOOTER: &str = "╚════════════════════════════════════════════════════════════════╝";

struct TestResult {
    name: String,
    passed: bool,
    duration_ms: f64,
    detail: String,
    conservation: bool,
}

impl TestResult {
    fn ok(name: &str, detail: &str, ms: f64, cons: bool) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            duration_ms: ms,
            detail: detail.to_string(),
            conservation: cons,
        }
    }
    fn fail(name: &str, detail: &str, ms: f64) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            duration_ms: ms,
            detail: detail.to_string(),
            conservation: false,
        }
    }
}

fn make_anchor(i: i32) -> Coord7D {
    let x = (i % 100) * 20;
    let y = ((i / 100) % 100) * 20;
    let z = (i / 10000) * 20;
    Coord7D::new_even([x, y, z, 0, 0, 0, 0])
}

fn make_data(i: i32, dim: usize) -> Vec<f64> {
    (0..dim)
        .map(|d| ((i * 7 + d as i32 * 13) as f64).sin() * 40.0)
        .collect()
}

fn main() {
    println!("{}", HEADER);
    println!("║   TetraMem-XL v12.0 极限压力测试 — 12维全覆盖            ║");
    println!("{}", FOOTER);
    println!();

    let mut results = Vec::new();
    let total_start = Instant::now();

    results.push(test_s1_massive_nodes());
    results.push(test_s2_memory_flood());
    results.push(test_s3_encode_decode_hammer());
    results.push(test_s4_erase_rewrite_cycles());
    results.push(test_s5_energy_transfer_storm());
    results.push(test_s6_dream_cycles());
    results.push(test_s7_full_pipeline());
    results.push(test_s8_persist_restore_loop());
    results.push(test_s9_regulation_under_load());
    results.push(test_s10_mixed_dimensions());
    results.push(test_s11_conservation_after_chaos());
    results.push(test_s12_longevity());

    let total_ms = total_start.elapsed().as_secs_f64() * 1000.0;
    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    let all_conserved = results.iter().all(|r| r.conservation);

    println!();
    println!("{}", SEPARATOR);
    println!("  极限压力测试报告");
    println!("{}", SEPARATOR);
    for r in &results {
        let mark = if r.passed { "✓" } else { "✗" };
        let cons = if r.conservation { "守恒✓" } else { "" };
        println!(
            "  {} {:<45} {:>6.1}ms  {} {}",
            mark, r.name, r.duration_ms, r.detail, cons
        );
    }
    println!("{}", SEPARATOR);
    println!(
        "  结果: {}/{} 通过  总耗时: {:.0}ms  全程守恒: {}",
        passed,
        total,
        total_ms,
        if all_conserved { "✓" } else { "✗" }
    );
    println!("{}", SEPARATOR);

    if passed < total {
        std::process::exit(1);
    }
}

fn test_s1_massive_nodes() -> TestResult {
    let name = "S1: 百万能量 × 10K节点 × 100K操作";
    println!("━━━ {} ━━━", name);
    let t = Instant::now();

    let mut u = DarkUniverse::new(1_000_000.0);
    let mut rng_state: u64 = 12345;
    let mut nodes = Vec::new();

    let target = 10_000;
    for i in 0..target {
        let x = i % 100;
        let y = (i / 100) % 100;
        let z = i / 10000;
        let c = Coord7D::new_even([x, y, z, 0, 0, 0, 0]);
        let amount = 10.0 + (i as f64 % 80.0);
        if u.materialize_biased(c, amount, 0.6).is_ok() {
            nodes.push(c);
        }
    }
    let created = nodes.len();

    let mut flow_count = 0usize;
    let mut transfer_count = 0usize;
    for _ in 0..100_000 {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let idx = (rng_state as usize) % nodes.len();
        let coord = nodes[idx];
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let op = (rng_state as usize) % 3;
        match op {
            0 => {
                let node = u.get_node(&coord);
                let avail = node.map_or(0.0, |n| n.energy().physical().min(0.5));
                if avail > 0.01 && u.flow_node_physical_to_dark(&coord, avail * 0.1).is_ok() {
                    flow_count += 1;
                }
            }
            1 => {
                let node = u.get_node(&coord);
                let avail = node.map_or(0.0, |n| n.energy().dark().min(0.5));
                if avail > 0.01 && u.flow_node_dark_to_physical(&coord, avail * 0.1).is_ok() {
                    flow_count += 1;
                }
            }
            _ => {
                let idx2 = ((rng_state as usize) + 1) % nodes.len();
                if idx != idx2 && u.transfer_energy(&nodes[idx], &nodes[idx2], 0.001).is_ok() {
                    transfer_count += 1;
                }
            }
        }
    }

    let cons = u.verify_conservation();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let detail = format!(
        "节点{} flow{} transfer{}",
        created, flow_count, transfer_count
    );
    println!("  {} {:.0}ms", detail, ms);
    TestResult::ok(name, &detail, ms, cons)
}

fn test_s2_memory_flood() -> TestResult {
    let name = "S2: 万轮记忆洪泛(自动扩展)";
    println!("━━━ {} ━━━", name);
    let t = Instant::now();

    let rounds = 10_000;
    let mut u = DarkUniverse::new(50_000.0);
    let scaler = AutoScaler::new();
    let mut mems = Vec::new();
    let mut datasets = Vec::new();
    let mut _encode_fails = 0;
    let mut expand_events = 0;

    for i in 0..rounds {
        let data = make_data(i, 7);
        let anchor = make_anchor(i);
        match MemoryCodec::encode(&mut u, &anchor, &data) {
            Ok(mem) => {
                mems.push(mem);
                datasets.push(data);
            }
            Err(_) => {
                let _ = scaler.scale_near_anchor(&mut u, &anchor, &data);
                expand_events += 1;
                match MemoryCodec::encode(&mut u, &anchor, &data) {
                    Ok(mem) => {
                        mems.push(mem);
                        datasets.push(data);
                    }
                    Err(_) => _encode_fails += 1,
                }
            }
        }
    }

    let mut max_err = 0.0f64;
    let mut corrupt = 0;
    for (i, mem) in mems.iter().enumerate() {
        if let Ok(decoded) = MemoryCodec::decode(&u, mem) {
            let err = datasets[i]
                .iter()
                .zip(decoded.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f64, f64::max);
            if err > 1e-10 {
                corrupt += 1;
            }
            max_err = max_err.max(err);
        } else {
            corrupt += 1;
        }
    }

    let cons = u.verify_conservation();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let detail = format!(
        "{}/{}记忆 精度{:.2e} 损坏{} 扩展{}次 能量{:.0}",
        mems.len(),
        rounds,
        max_err,
        corrupt,
        expand_events,
        u.total_energy()
    );
    println!("  {} {:.0}ms", detail, ms);

    if corrupt > 0 || !cons {
        return TestResult::fail(name, &format!("corrupt={} cons={}", corrupt, cons), ms);
    }
    TestResult::ok(name, &detail, ms, cons)
}

fn test_s3_encode_decode_hammer() -> TestResult {
    let name = "S3: 1K记忆×1000次decode锤击";
    println!("━━━ {} ━━━", name);
    let t = Instant::now();

    let mem_count = 1_000;
    let decode_rounds = 1_000;
    let mut u = DarkUniverse::new(5_000_000.0);
    let mut mems = Vec::new();
    let mut datasets = Vec::new();

    for i in 0..mem_count {
        let data = make_data(i, 14);
        let anchor = make_anchor(i);
        match MemoryCodec::encode(&mut u, &anchor, &data) {
            Ok(mem) => {
                mems.push(mem);
                datasets.push(data);
            }
            Err(_) => continue,
        }
    }

    let mut max_err = 0.0f64;
    let mut total_decodes = 0;
    for _ in 0..decode_rounds {
        for (i, mem) in mems.iter().enumerate() {
            if let Ok(decoded) = MemoryCodec::decode(&u, mem) {
                total_decodes += 1;
                let err = datasets[i]
                    .iter()
                    .zip(decoded.iter())
                    .map(|(a, b)| (a - b).abs())
                    .fold(0.0f64, f64::max);
                max_err = max_err.max(err);
            }
        }
    }

    let cons = u.verify_conservation();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let detail = format!(
        "编码{} decode{}×{} 精度{:.2e}",
        mems.len(),
        decode_rounds,
        mems.len(),
        max_err
    );
    println!("  {} {:.0}ms", detail, ms);
    TestResult::ok(
        name,
        &format!("{}次decode 精度{:.2e}", total_decodes, max_err),
        ms,
        cons,
    )
}

fn test_s4_erase_rewrite_cycles() -> TestResult {
    let name = "S4: 5000次 erase→rewrite 完整循环";
    println!("━━━ {} ━━━", name);
    let t = Instant::now();

    let cycles = 5_000;
    let mut u = DarkUniverse::new(1_000_000.0);
    let mut max_err = 0.0f64;
    let mut successful = 0;

    for i in 0..cycles {
        let data = vec![
            (i as f64 * 1.1).sin() * 40.0,
            (i as f64 * 0.7).cos() * 40.0,
            (i as f64).sin() * 40.0,
        ];
        let anchor = Coord7D::new_even([(i % 200) * 5, 0, 0, 0, 0, 0, 0]);
        if let Ok(mem) = MemoryCodec::encode(&mut u, &anchor, &data) {
            if let Ok(decoded) = MemoryCodec::decode(&u, &mem) {
                let err = data
                    .iter()
                    .zip(decoded.iter())
                    .map(|(a, b)| (a - b).abs())
                    .fold(0.0f64, f64::max);
                max_err = max_err.max(err);
            }
            MemoryCodec::erase(&mut u, &mem);
            successful += 1;
        }
    }

    let cons = u.verify_conservation();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let detail = format!("{}/{}循环 精度{:.2e}", successful, cycles, max_err);
    println!("  {} {:.0}ms", detail, ms);

    if !cons {
        let stats = u.stats();
        let node_total: f64 = u.get_all_nodes().values().map(|n| n.energy().total()).sum();
        let diff = (node_total - stats.allocated_energy).abs();
        println!(
            "  S4守恒详情: allocated={:.6} node_total={:.6} diff={:.2e}",
            stats.allocated_energy, node_total, diff
        );
        return TestResult::fail(name, &format!("守恒违反 diff={:.2e}", diff), ms);
    }
    TestResult::ok(name, &detail, ms, cons)
}

fn test_s5_energy_transfer_storm() -> TestResult {
    let name = "S5: 50K节点间能量迁移风暴";
    println!("━━━ {} ━━━", name);
    let t = Instant::now();

    let mut u = DarkUniverse::new(1_000_000.0);
    let node_count = 500;
    let transfers = 50_000;
    let mut nodes = Vec::new();

    for i in 0..node_count {
        let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
        u.materialize_uniform(c, 100.0).ok();
        nodes.push(c);
    }
    let before_avail = u.available_energy();

    let mut rng_state: u64 = 98765;
    let mut successful_transfers = 0;
    for _ in 0..transfers {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let i = (rng_state as usize) % nodes.len();
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let j = (rng_state as usize) % nodes.len();
        if i != j && u.transfer_energy(&nodes[i], &nodes[j], 0.5).is_ok() {
            successful_transfers += 1;
        }
    }

    let after_avail = u.available_energy();
    let cons = u.verify_conservation();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let avail_diff = (after_avail - before_avail).abs();
    let detail = format!("成功{} 可用能差{:.2e}", successful_transfers, avail_diff);
    println!("  {} {:.0}ms", detail, ms);

    if !cons || avail_diff > 1e-6 {
        return TestResult::fail(
            name,
            &format!("cons={} avail_diff={:.2e}", cons, avail_diff),
            ms,
        );
    }
    TestResult::ok(name, &detail, ms, cons)
}

fn test_s6_dream_cycles() -> TestResult {
    let name = "S6: 100轮梦境循环(500记忆)";
    println!("━━━ {} ━━━", name);
    let t = Instant::now();

    let mut u = DarkUniverse::new(10_000_000.0);
    let mut hebbian = HebbianMemory::new();
    let mut mems = Vec::new();

    for i in 0..500 {
        let data = make_data(i, 7);
        let anchor = make_anchor(i);
        if let Ok(mem) = MemoryCodec::encode(&mut u, &anchor, &data) {
            let pulse = PulseEngine::new();
            for v in mem.vertices().iter() {
                let _ = pulse.propagate(v, PulseType::Reinforcing, &u, &mut hebbian);
            }
            let verts = mem.vertices();
            for w in 1..4 {
                hebbian.record_path(&[verts[0], verts[w]], 1.0);
            }
            mems.push(mem);
        }
    }

    let dream = DreamEngine::new();
    let mut total_replayed = 0;
    let mut total_weakened = 0;
    let mut total_consolidated = 0;

    for _ in 0..100 {
        let report = dream.dream(&u, &mut hebbian, &mems);
        total_replayed += report.paths_replayed;
        total_weakened += report.paths_weakened;
        total_consolidated += report.memories_consolidated;
    }

    let cons = u.verify_conservation();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let detail = format!(
        "记忆{} 赫布{} replay:{} weaken:{} consol:{}",
        mems.len(),
        hebbian.edge_count(),
        total_replayed,
        total_weakened,
        total_consolidated
    );
    println!("  {} {:.0}ms", detail, ms);
    TestResult::ok(name, &detail, ms, cons)
}

fn test_s7_full_pipeline() -> TestResult {
    let name = "S7: 闭环流水线(encode→pulse→hebbian→crystal→dream→regulate×50轮)";
    println!("━━━ {} ━━━", name);
    let t = Instant::now();

    let pipeline_rounds = 50;
    let mut u = DarkUniverse::new(5_000_000.0);
    let mut hebbian = HebbianMemory::new();
    let mut crystal = CrystalEngine::new();
    let mut mems = Vec::new();
    let mut datasets = Vec::new();
    let scaler = AutoScaler::new();
    let reg = RegulationEngine::new();
    let mut total_actions = 0;

    for round in 0..pipeline_rounds {
        let base_idx = round * 20;
        for j in 0..20 {
            let i = base_idx + j;
            let data = make_data(i, 14);
            let anchor = make_anchor(i);
            match MemoryCodec::encode(&mut u, &anchor, &data) {
                Ok(mem) => {
                    let pulse = PulseEngine::new();
                    for v in mem.vertices().iter() {
                        let _ = pulse.propagate(v, PulseType::Reinforcing, &u, &mut hebbian);
                    }
                    mems.push(mem);
                    datasets.push(data);
                }
                Err(_) => {
                    let _ = scaler.scale_near_anchor(&mut u, &anchor, &data);
                }
            }
        }

        let _ = crystal.crystallize(&hebbian, &u);

        if round % 10 == 9 {
            let dream = DreamEngine::new();
            let _ = dream.dream(&u, &mut hebbian, &mems);
        }

        let reg_report = reg.regulate(&mut u, &mut hebbian, &mut crystal, &mems);
        total_actions += reg_report.actions.len();
    }

    let mut max_err = 0.0f64;
    for (i, mem) in mems.iter().enumerate() {
        if let Ok(decoded) = MemoryCodec::decode(&u, mem) {
            let err = datasets[i]
                .iter()
                .zip(decoded.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f64, f64::max);
            max_err = max_err.max(err);
        }
    }

    let cons = u.verify_conservation();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let detail = format!(
        "记忆{} 赫布{} 结晶{} 调控动作{} 精度{:.2e}",
        mems.len(),
        hebbian.edge_count(),
        crystal.channel_count(),
        total_actions,
        max_err
    );
    println!("  {} {:.0}ms", detail, ms);
    TestResult::ok(name, &detail, ms, cons)
}

fn test_s8_persist_restore_loop() -> TestResult {
    let name = "S8: 100次 持久化→恢复→验证 循环";
    println!("━━━ {} ━━━", name);
    let t = Instant::now();

    let cycles = 100;
    let mut u = DarkUniverse::new(500_000.0);
    let mut hebbian = HebbianMemory::new();
    let mut crystal = CrystalEngine::new();
    let mut mems = Vec::new();
    let mut datasets = Vec::new();

    for i in 0..50 {
        let data = make_data(i, 7);
        let anchor = make_anchor(i);
        if let Ok(mem) = MemoryCodec::encode(&mut u, &anchor, &data) {
            let verts = mem.vertices();
            for w in 1..4 {
                hebbian.record_path(&[verts[0], verts[w]], 1.5);
            }
            mems.push(mem);
            datasets.push(data);
        }
    }
    crystal.crystallize(&hebbian, &u);

    let mut max_err = 0.0f64;
    for _ in 0..cycles {
        let json = PersistEngine::to_json(&u, &hebbian, &mems, &crystal).unwrap();
        let (u2, h2, m2, c2) = PersistEngine::from_json(&json).unwrap();

        for (i, mem) in m2.iter().enumerate() {
            if let Ok(decoded) = MemoryCodec::decode(&u2, mem) {
                let err = datasets[i]
                    .iter()
                    .zip(decoded.iter())
                    .map(|(a, b)| (a - b).abs())
                    .fold(0.0f64, f64::max);
                max_err = max_err.max(err);
            }
        }

        if !u2.verify_conservation() {
            return TestResult::fail(name, "恢复后守恒违反", t.elapsed().as_secs_f64() * 1000.0);
        }
        if h2.edge_count() != hebbian.edge_count() {
            return TestResult::fail(name, "赫布边数不匹配", t.elapsed().as_secs_f64() * 1000.0);
        }
        if c2.channel_count() != crystal.channel_count() {
            return TestResult::fail(name, "结晶数不匹配", t.elapsed().as_secs_f64() * 1000.0);
        }
    }

    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let detail = format!(
        "{}次roundtrip 赫布{} 结晶{} 精度{:.2e}",
        cycles,
        hebbian.edge_count(),
        crystal.channel_count(),
        max_err
    );
    println!("  {} {:.0}ms", detail, ms);
    TestResult::ok(name, &detail, ms, true)
}

fn test_s9_regulation_under_load() -> TestResult {
    let name = "S9: 高负载下200轮调控";
    println!("━━━ {} ━━━", name);
    let t = Instant::now();

    let rounds = 200;
    let mut u = DarkUniverse::new(200_000.0);
    let mut hebbian = HebbianMemory::new();
    let mut crystal = CrystalEngine::new();
    let mut mems = Vec::new();
    let scaler = AutoScaler::new();
    let reg = RegulationEngine::new();
    let mut reg_actions_total = 0;

    for round in 0..rounds {
        let data = make_data(round, 7);
        let anchor = make_anchor(round);
        match MemoryCodec::encode(&mut u, &anchor, &data) {
            Ok(mem) => {
                let verts = mem.vertices();
                hebbian.record_path(&[verts[0], verts[1]], 1.0);
                mems.push(mem);
            }
            Err(_) => {
                let _ = scaler.scale_near_anchor(&mut u, &anchor, &data);
            }
        }

        if round % 5 == 4 {
            let report = reg.regulate(&mut u, &mut hebbian, &mut crystal, &mems);
            reg_actions_total += report.actions.len();
        }

        if round % 20 == 19 {
            crystal.crystallize(&hebbian, &u);
        }
    }

    let cons = u.verify_conservation();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let detail = format!(
        "记忆{} 赫布{} 结晶{} 调控动作{}",
        mems.len(),
        hebbian.edge_count(),
        crystal.channel_count(),
        reg_actions_total
    );
    println!("  {} {:.0}ms", detail, ms);
    TestResult::ok(name, &detail, ms, cons)
}

fn test_s10_mixed_dimensions() -> TestResult {
    let name = "S10: 1~28维混合记忆共存2000个";
    println!("━━━ {} ━━━", name);
    let t = Instant::now();

    let mut u = DarkUniverse::new(20_000_000.0);
    let mut mems = Vec::new();
    let mut datasets = Vec::new();
    let scaler = AutoScaler::new();
    let mut dim_counts: HashMap<usize, usize> = HashMap::new();

    for i in 0..2000 {
        let dim = (i % 28) + 1;
        let data = make_data(i as i32, dim);
        let anchor = make_anchor(i as i32);
        match MemoryCodec::encode(&mut u, &anchor, &data) {
            Ok(mem) => {
                mems.push(mem);
                datasets.push(data);
                *dim_counts.entry(dim).or_insert(0) += 1;
            }
            Err(_) => {
                let _ = scaler.scale_near_anchor(&mut u, &anchor, &data);
                if let Ok(mem) = MemoryCodec::encode(&mut u, &anchor, &data) {
                    mems.push(mem);
                    datasets.push(data);
                    *dim_counts.entry(dim).or_insert(0) += 1;
                }
            }
        }
    }

    let mut max_err = 0.0f64;
    let mut dim_max_errs: HashMap<usize, f64> = HashMap::new();
    for (i, mem) in mems.iter().enumerate() {
        if let Ok(decoded) = MemoryCodec::decode(&u, mem) {
            let err = datasets[i]
                .iter()
                .zip(decoded.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f64, f64::max);
            let dim = mem.data_dim();
            let entry = dim_max_errs.entry(dim).or_insert(0.0);
            *entry = entry.max(err);
            max_err = max_err.max(err);
        }
    }

    let cons = u.verify_conservation();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let detail = format!(
        "{}/2000记忆 精度{:.2e} 覆盖{}种维度",
        mems.len(),
        max_err,
        dim_counts.len()
    );
    println!("  {} {:.0}ms", detail, ms);

    if max_err > 1e-10 {
        return TestResult::fail(name, &format!("精度退化: {:.2e}", max_err), ms);
    }
    TestResult::ok(name, &detail, ms, cons)
}

fn test_s11_conservation_after_chaos() -> TestResult {
    let name = "S11: 混沌操作后守恒(随机5万次混合操作)";
    println!("━━━ {} ━━━", name);
    let t = Instant::now();

    let mut u = DarkUniverse::new(2_000_000.0);
    let mut nodes = Vec::new();
    let mut mems = Vec::new();
    let mut rng_state: u64 = 42;

    for i in 0..500 {
        let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
        u.materialize_biased(c, 100.0, 0.5).ok();
        nodes.push(c);
    }

    for i in 0..100 {
        let data = make_data(i, 7);
        let anchor = Coord7D::new_even([500 + i, 0, 0, 0, 0, 0, 0]);
        if let Ok(mem) = MemoryCodec::encode(&mut u, &anchor, &data) {
            mems.push(mem);
        }
    }

    let chaos_ops = 50_000;
    let rng = |state: &mut u64| {
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        *state
    };
    let mut flow_ok = 0u64;
    let mut transfer_ok = 0u64;
    let mut demat_ok = 0u64;

    for _ in 0..chaos_ops {
        rng(&mut rng_state);
        let op = (rng_state as usize) % 5;
        match op {
            0 => {
                if !nodes.is_empty() {
                    rng(&mut rng_state);
                    let idx = (rng_state as usize) % nodes.len();
                    rng(&mut rng_state);
                    let dir = (rng_state as usize) % 2;
                    let result = if dir == 0 {
                        u.flow_node_physical_to_dark(&nodes[idx], 1.0)
                    } else {
                        u.flow_node_dark_to_physical(&nodes[idx], 1.0)
                    };
                    if result.is_ok() {
                        flow_ok += 1;
                    }
                }
            }
            1 => {
                if nodes.len() >= 2 {
                    rng(&mut rng_state);
                    let i = (rng_state as usize) % nodes.len();
                    rng(&mut rng_state);
                    let j = (rng_state as usize) % nodes.len();
                    if i != j && u.transfer_energy(&nodes[i], &nodes[j], 0.5).is_ok() {
                        transfer_ok += 1;
                    }
                }
            }
            2 => {
                if !mems.is_empty() {
                    rng(&mut rng_state);
                    let idx = (rng_state as usize) % mems.len();
                    let mem = mems.swap_remove(idx);
                    MemoryCodec::erase(&mut u, &mem);
                    demat_ok += 1;
                }
            }
            3 => {
                rng(&mut rng_state);
                let x = 600 + (rng_state as i32 % 1000);
                let c = Coord7D::new_even([x, 0, 0, 0, 0, 0, 0]);
                u.expand_energy_pool(500.0);
                if u.materialize_biased(c, 50.0, 0.6).is_ok() {
                    nodes.push(c);
                }
            }
            _ => {
                u.expand_energy_pool(100.0);
            }
        }
    }

    let cons = u.verify_conservation();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let detail = format!(
        "flow{} transfer{} erase{} 节点{} 记忆{} 能量{:.0}",
        flow_ok,
        transfer_ok,
        demat_ok,
        nodes.len(),
        mems.len(),
        u.total_energy()
    );
    println!("  {} {:.0}ms", detail, ms);

    if !cons {
        return TestResult::fail(name, "守恒违反", ms);
    }
    TestResult::ok(name, &detail, ms, cons)
}

fn test_s12_longevity() -> TestResult {
    let name = "S12: 长寿测试(20K轮 encode+decode+verify)";
    println!("━━━ {} ━━━", name);
    let t = Instant::now();

    let rounds = 20_000;
    let mut u = DarkUniverse::new(50_000_000.0);
    let mut mems = Vec::new();
    let mut datasets = Vec::new();
    let mut max_err = 0.0f64;
    let mut corrupt = 0;
    let mut verify_checkpoints = 0;

    for i in 0..rounds {
        let dim = (i % 28) + 1;
        let data = make_data(i as i32, dim);
        let anchor = make_anchor(i as i32);
        match MemoryCodec::encode(&mut u, &anchor, &data) {
            Ok(mem) => {
                mems.push(mem);
                datasets.push(data);
            }
            Err(_) => continue,
        }

        if i > 0 && i % 5000 == 0 {
            if !u.verify_conservation() {
                let ms = t.elapsed().as_secs_f64() * 1000.0;
                return TestResult::fail(name, &format!("轮{}守恒违反", i), ms);
            }
            verify_checkpoints += 1;
        }
    }

    for (i, mem) in mems.iter().enumerate() {
        if let Ok(decoded) = MemoryCodec::decode(&u, mem) {
            let err = datasets[i]
                .iter()
                .zip(decoded.iter())
                .map(|(a, b)| (a - b).abs())
                .fold(0.0f64, f64::max);
            if err > 1e-10 {
                corrupt += 1;
            }
            max_err = max_err.max(err);
        } else {
            corrupt += 1;
        }
    }

    let cons = u.verify_conservation();
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let detail = format!(
        "{}/{}记忆 精度{:.2e} 损坏{} 守恒检查{}次 节点{} 能量{:.0}",
        mems.len(),
        rounds,
        max_err,
        corrupt,
        verify_checkpoints,
        u.active_node_count(),
        u.total_energy()
    );
    println!("  {} {:.0}ms", detail, ms);

    if corrupt > 0 || !cons {
        return TestResult::fail(name, &format!("corrupt={} cons={}", corrupt, cons), ms);
    }
    TestResult::ok(name, &detail, ms, cons)
}
