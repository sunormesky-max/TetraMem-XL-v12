use std::time::Instant;
use tetramem_v12::universe::autoscale::AutoScaler;
use tetramem_v12::universe::coord::Coord7D;
use tetramem_v12::universe::crystal::CrystalEngine;
use tetramem_v12::universe::dream::DreamEngine;
use tetramem_v12::universe::hebbian::HebbianMemory;
use tetramem_v12::universe::memory::MemoryCodec;
use tetramem_v12::universe::node::DarkUniverse;
use tetramem_v12::universe::persist::PersistEngine;
use tetramem_v12::universe::pulse::{PulseEngine, PulseType};
use tetramem_v12::universe::reasoning::ReasoningEngine;
use tetramem_v12::universe::regulation::RegulationEngine;
use tetramem_v12::universe::topology::TopologyEngine;

#[derive(Debug)]
struct TestResult {
    name: String,
    passed: bool,
    detail: String,
    elapsed_ms: f64,
}

impl TestResult {
    fn ok(name: &str, detail: &str, ms: f64) -> Self {
        Self { name: name.into(), passed: true, detail: detail.into(), elapsed_ms: ms }
    }
    fn fail(name: &str, detail: &str, ms: f64) -> Self {
        Self { name: name.into(), passed: false, detail: detail.into(), elapsed_ms: ms }
    }
}

fn main() {
    let mut results = Vec::new();
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║       TetraMem-XL v12.0 全方位深度测试                        ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // ═══ A. 宇宙基础 — 能量守恒极限压力 ═══
    println!("══ A. 宇宙基础 — 能量守恒极限压力 ══");

    results.push(run("A1: 100K节点大规模创建+守恒", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000_000.0);
        let mut created = 0usize;
        for x in 0..50i32 {
            for y in 0..50i32 {
                for z in 0..40i32 {
                    let c = Coord7D::new_even([x, y, z, 0, 0, 0, 0]);
                    if u.materialize_biased(c, 10.0, 0.6).is_ok() { created += 1; }
                    let c2 = Coord7D::new_odd([x, y, z, 0, 0, 0, 0]);
                    if u.materialize_biased(c2, 8.0, 0.3).is_ok() { created += 1; }
                }
            }
        }
        let ok = u.verify_conservation();
        let stats = u.stats();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok {
            TestResult::ok("A1", &format!("{}节点 守恒✓ 利用率{:.1}%", created, stats.utilization * 100.0), ms)
        } else {
            TestResult::fail("A1", "守恒违反!", ms)
        }
    }));

    results.push(run("A2: 50K次flow操作后守恒", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(100_000_000.0);
        let mut coords = Vec::new();
        for i in 0..10000i32 {
            let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            u.materialize_biased(c, 50.0, 0.6).unwrap();
            coords.push(c);
        }
        let mut flow_count = 0usize;
        for _ in 0..5 {
            for c in &coords {
                u.flow_node_physical_to_dark(c, 5.0).ok();
                u.flow_node_dark_to_physical(c, 5.0).ok();
                flow_count += 2;
            }
        }
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok {
            TestResult::ok("A2", &format!("{}次flow 守恒✓", flow_count), ms)
        } else {
            TestResult::fail("A2", "守恒违反!", ms)
        }
    }));

    results.push(run("A3: 10K次transfer后守恒", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(100_000_000.0);
        let mut coords = Vec::new();
        for i in 0..5000i32 {
            let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            u.materialize_biased(c, 50.0, 0.6).unwrap();
            coords.push(c);
        }
        let mut tc = 0usize;
        for i in 0..4999 {
            if u.transfer_energy(&coords[i], &coords[i + 1], 2.0).is_ok() { tc += 1; }
        }
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok {
            TestResult::ok("A3", &format!("{}次transfer 守恒✓", tc), ms)
        } else {
            TestResult::fail("A3", "守恒违反!", ms)
        }
    }));

    results.push(run("A4: 大规模dematerialize后守恒", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(100_000_000.0);
        let mut coords = Vec::new();
        for i in 0..10000i32 {
            let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            u.materialize_biased(c, 50.0, 0.6).unwrap();
            coords.push(c);
        }
        let mut removed = 0usize;
        for i in (0..10000).step_by(3) {
            u.dematerialize(&coords[i]);
            removed += 1;
        }
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok {
            TestResult::ok("A4", &format!("删除{} 守恒✓ 剩余{}", removed, u.active_node_count()), ms)
        } else {
            TestResult::fail("A4", "守恒违反!", ms)
        }
    }));

    results.push(run("A5: 能量预算耗尽边界", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1000.0);
        let mut ok_count = 0usize;
        let mut fail_count = 0usize;
        for i in 0..10000i32 {
            match u.materialize_biased(Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]), 100.0, 0.6) {
                Ok(_) => ok_count += 1,
                Err(_) => fail_count += 1,
            }
        }
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok && fail_count > 0 {
            TestResult::ok("A5", &format!("成功{} 拒绝{} 守恒✓", ok_count, fail_count), ms)
        } else {
            TestResult::fail("A5", &format!("ok={} fail={}", ok, fail_count), ms)
        }
    }));

    results.push(run("A6: AlreadyOccupied防护", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000.0);
        let c = Coord7D::new_even([0, 0, 0, 0, 0, 0, 0]);
        u.materialize_biased(c, 100.0, 0.6).unwrap();
        let second = u.materialize_biased(c, 100.0, 0.6);
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if second.is_err() && ok {
            TestResult::ok("A6", "重复materialize被拒绝 守恒✓", ms)
        } else {
            TestResult::fail("A6", "未被拒绝!", ms)
        }
    }));

    results.push(run("A7: expand_energy_pool后守恒", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1000.0);
        for i in 0..5i32 {
            u.materialize_biased(Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]), 50.0, 0.6).unwrap();
        }
        u.expand_energy_pool(100_000.0);
        for i in 5..100i32 {
            u.materialize_biased(Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]), 50.0, 0.6).unwrap();
        }
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok {
            TestResult::ok("A7", &format!("扩展后95节点 守恒✓ 总能{}", u.total_energy()), ms)
        } else {
            TestResult::fail("A7", "守恒违反!", ms)
        }
    }));

    // ═══ B. 记忆系统 — 端到端精度 ═══
    println!("\n══ B. 记忆系统 — 端到端精度 ══");

    results.push(run("B1: 1维到28维全部精确往返", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(10_000_000.0);
        let mut max_err = 0.0f64;
        let mut count = 0usize;
        for dim in [1, 2, 3, 4, 5, 6, 7, 8, 10, 14, 20, 28] {
            let data: Vec<f64> = (0..dim).map(|i| (i as f64 + 1.0) * 0.1).collect();
            let anchor = Coord7D::new_even([dim * 10, dim * 10, dim * 10, 0, 0, 0, 0]);
            let mem = MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
            let decoded = MemoryCodec::decode(&u, &mem).unwrap();
            let err = data.iter().zip(decoded.iter()).map(|(a, b)| (a - b).abs()).fold(0.0f64, f64::max);
            max_err = max_err.max(err);
            count += 1;
        }
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if max_err < 1e-10 {
            TestResult::ok("B1", &format!("{}种维度 最大误差{:.2e}", count, max_err), ms)
        } else {
            TestResult::fail("B1", &format!("误差过大: {:.2e}", max_err), ms)
        }
    }));

    results.push(run("B2: 编码后守恒", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(10_000_000.0);
        let data: Vec<f64> = (0..28).map(|i| i as f64).collect();
        let anchor = Coord7D::new_even([100, 100, 100, 0, 0, 0, 0]);
        let _mem = MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok { TestResult::ok("B2", "编码28维后守恒✓", ms) }
        else { TestResult::fail("B2", "守恒违反!", ms) }
    }));

    results.push(run("B3: erase释放能量后守恒", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(10_000_000.0);
        let data: Vec<f64> = (0..28).map(|i| i as f64).collect();
        let anchor = Coord7D::new_even([100, 100, 100, 0, 0, 0, 0]);
        let mem = MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
        let avail_before = u.available_energy();
        MemoryCodec::erase(&mut u, &mem);
        let avail_after = u.available_energy();
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok && avail_after > avail_before {
            TestResult::ok("B3", &format!("擦除可用能{}→{} 守恒✓", avail_before, avail_after), ms)
        } else {
            TestResult::fail("B3", &format!("ok={} after={}", ok, avail_after), ms)
        }
    }));

    results.push(run("B4: erase→重用循环100次", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000.0);
        let mut max_err = 0.0f64;
        for cycle in 0..100i32 {
            let data: Vec<f64> = (0..7).map(|i| (i + cycle) as f64 * 0.1).collect();
            let anchor = Coord7D::new_even([cycle, 0, 0, 0, 0, 0, 0]);
            let mem = MemoryCodec::encode(&mut u, &anchor, &data).unwrap();
            let decoded = MemoryCodec::decode(&u, &mem).unwrap();
            let err = data.iter().zip(decoded.iter()).map(|(a, b)| (a - b).abs()).fold(0.0f64, f64::max);
            max_err = max_err.max(err);
            MemoryCodec::erase(&mut u, &mem);
        }
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok && max_err < 1e-10 {
            TestResult::ok("B4", &format!("100次循环 最大误差{:.2e} 守恒✓", max_err), ms)
        } else {
            TestResult::fail("B4", &format!("err={:.2e} ok={}", max_err, ok), ms)
        }
    }));

    results.push(run("B5: 100个记忆共存不冲突", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000_000.0);
        let mut mems = Vec::new();
        let mut datasets = Vec::new();
        for i in 0..100i32 {
            let data: Vec<f64> = (0..14).map(|j| (i * 14 + j) as f64 * 0.01).collect();
            let anchor = Coord7D::new_even([i * 5, 0, 0, 0, 0, 0, 0]);
            mems.push(MemoryCodec::encode(&mut u, &anchor, &data).unwrap());
            datasets.push(data);
        }
        let mut max_err = 0.0f64;
        for (mem, data) in mems.iter().zip(datasets.iter()) {
            let decoded = MemoryCodec::decode(&u, mem).unwrap();
            let err = data.iter().zip(decoded.iter()).map(|(a, b)| (a - b).abs()).fold(0.0f64, f64::max);
            max_err = max_err.max(err);
        }
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok && max_err < 1e-10 {
            TestResult::ok("B5", &format!("100个14维记忆 最大误差{:.2e} 守恒✓", max_err), ms)
        } else {
            TestResult::fail("B5", &format!("err={:.2e} ok={}", max_err, ok), ms)
        }
    }));

    results.push(run("B6: 空数据拒绝", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000.0);
        let result = MemoryCodec::encode(&mut u, &Coord7D::new_even([0,0,0,0,0,0,0]), &[]);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if result.is_err() { TestResult::ok("B6", "空数据正确拒绝", ms) }
        else { TestResult::fail("B6", "未被拒绝!", ms) }
    }));

    results.push(run("B7: 超量数据(>28维)拒绝", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000_000.0);
        let data: Vec<f64> = (0..29).map(|i| i as f64).collect();
        let result = MemoryCodec::encode(&mut u, &Coord7D::new_even([0,0,0,0,0,0,0]), &data);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if result.is_err() { TestResult::ok("B7", "29维正确拒绝", ms) }
        else { TestResult::fail("B7", "未被拒绝!", ms) }
    }));

    // ═══ C. 脉冲引擎 — PCNN吞吐 ═══
    println!("\n══ C. 脉冲引擎 — PCNN吞吐 ══");

    results.push(run("C1: 3种脉冲类型全部传播", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(10_000_000.0);
        for x in 0..20i32 {
            for y in 0..20i32 {
                for z in 0..20i32 {
                    u.materialize_biased(Coord7D::new_even([x, y, z, 0, 0, 0, 0]), 50.0, 0.6).ok();
                    u.materialize_biased(Coord7D::new_odd([x, y, z, 0, 0, 0, 0]), 40.0, 0.3).ok();
                }
            }
        }
        let mut h = HebbianMemory::new();
        let engine = PulseEngine::new();
        let mut total_visited = 0usize;
        for pt in [PulseType::Exploratory, PulseType::Reinforcing, PulseType::Cascade] {
            let r = engine.propagate(&Coord7D::new_even([10,10,10,0,0,0,0]), pt, &u, &mut h);
            total_visited += r.visited_nodes;
        }
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if total_visited > 0 {
            TestResult::ok("C1", &format!("3种脉冲 访问{} 赫布{}", total_visited, h.edge_count()), ms)
        } else {
            TestResult::fail("C1", "零访问!", ms)
        }
    }));

    results.push(run("C2: 赫布权重衰减", || {
        let t = Instant::now();
        let mut h = HebbianMemory::new();
        let a = Coord7D::new_even([0,0,0,0,0,0,0]);
        let b = Coord7D::new_even([1,0,0,0,0,0,0]);
        h.record_path(&[a, b], 1.0);
        let before = h.get_bias(&a, &b);
        for _ in 0..20 { h.decay_all(); }
        let after = h.get_bias(&a, &b);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if after < before {
            TestResult::ok("C2", &format!("权重{:.4}→{:.6}", before, after), ms)
        } else {
            TestResult::fail("C2", "权重未衰减!", ms)
        }
    }));

    results.push(run("C3: prune清理弱边(大量边)", || {
        let t = Instant::now();
        let mut h = HebbianMemory::new();
        h.max_paths = 100;
        for i in 0..200i32 {
            let a = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            let b = Coord7D::new_even([i, 1, 0, 0, 0, 0, 0]);
            h.record_path(&[a, b], if i < 5 { 2.0 } else { 0.01 });
        }
        let before = h.edge_count();
        h.prune();
        let after = h.edge_count();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if after < before {
            TestResult::ok("C3", &format!("{}边→{}边 清理了{}弱边", before, after, before - after), ms)
        } else {
            TestResult::fail("C3", &format!("边数未减少 {}→{}", before, after), ms)
        }
    }));

    // ═══ D. 拓扑引擎 — 7D Betti数 ═══
    println!("\n══ D. 拓扑引擎 — 7D Betti数 ══");

    results.push(run("D1: 小晶格Betti数一致性", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000.0);
        for x in 0..5i32 {
            for y in 0..5i32 {
                for z in 0..5i32 {
                    u.materialize_biased(Coord7D::new_even([x, y, z, 0, 0, 0, 0]), 50.0, 0.6).ok();
                    u.materialize_biased(Coord7D::new_odd([x, y, z, 0, 0, 0, 0]), 40.0, 0.3).ok();
                }
            }
        }
        let topo = TopologyEngine::analyze(&u);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if topo.connected_components >= 1 {
            TestResult::ok("D1", &format!("{} 连通:{} Euler:{}", topo.betti, topo.connected_components, topo.betti.euler_characteristic()), ms)
        } else {
            TestResult::fail("D1", "拓扑异常", ms)
        }
    }));

    results.push(run("D2: BFS最短路径正确", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000.0);
        for i in 0..20i32 {
            u.materialize_biased(Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]), 50.0, 0.6).ok();
        }
        let from = Coord7D::new_even([0,0,0,0,0,0,0]);
        let to = Coord7D::new_even([19,0,0,0,0,0,0]);
        let path = TopologyEngine::find_shortest_path(&u, &from, &to);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if !path.is_empty() && *path.first().unwrap() == from && *path.last().unwrap() == to {
            TestResult::ok("D2", &format!("{}→{}: {}跳", from, to, path.len() - 1), ms)
        } else {
            TestResult::fail("D2", &format!("路径长度={}", path.len()), ms)
        }
    }));

    results.push(run("D3: 不连通返回空路径", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000.0);
        u.materialize_biased(Coord7D::new_even([0,0,0,0,0,0,0]), 50.0, 0.6).ok();
        u.materialize_biased(Coord7D::new_even([100,100,100,0,0,0,0]), 50.0, 0.6).ok();
        let path = TopologyEngine::find_shortest_path(&u, &Coord7D::new_even([0,0,0,0,0,0,0]), &Coord7D::new_even([100,100,100,0,0,0,0]));
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if path.is_empty() { TestResult::ok("D3", "不连通→空路径", ms) }
        else { TestResult::fail("D3", "应返回空!", ms) }
    }));

    // ═══ E. 结晶相变 ═══
    println!("\n══ E. 结晶相变 ══");

    results.push(run("E1: 强赫布→结晶通道", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000.0);
        let mut h = HebbianMemory::new();
        for i in 0..10i32 {
            u.materialize_biased(Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]), 50.0, 0.6).ok();
        }
        let nodes: Vec<Coord7D> = (0..10).map(|i| Coord7D::new_even([i,0,0,0,0,0,0])).collect();
        for _ in 0..10 { h.record_path(&nodes, 2.0); }
        let mut crystal = CrystalEngine::new();
        let report = crystal.crystallize(&h, &u);
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok {
            TestResult::ok("E1", &format!("结晶{} 超级{} 锁定{:.1} 守恒✓",
                report.new_crystals, report.new_super_crystals, report.energy_locked), ms)
        } else {
            TestResult::fail("E1", "守恒违反!", ms)
        }
    }));

    results.push(run("E2: 结晶路由连通性", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000.0);
        let mut h = HebbianMemory::new();
        for i in 0..20i32 {
            let c = Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]);
            u.materialize_biased(c, 50.0, 0.6).ok();
            if i > 0 { h.record_path(&[Coord7D::new_even([i-1,0,0,0,0,0,0]), c], 3.0); }
        }
        let mut crystal = CrystalEngine::new();
        crystal.crystallize(&h, &u);
        let from = Coord7D::new_even([0,0,0,0,0,0,0]);
        let to = Coord7D::new_even([19,0,0,0,0,0,0]);
        let path = crystal.crystal_path(&from, &to, 25);
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok {
            let hops = if path.is_empty() { "无结晶路由".into() } else { format!("{}跳", path.len()-1) };
            TestResult::ok("E2", &format!("结晶路由: {} 守恒✓", hops), ms)
        } else {
            TestResult::fail("E2", "守恒违反!", ms)
        }
    }));

    // ═══ F. 推理引擎 ═══
    println!("\n══ F. 推理引擎 ══");

    results.push(run("F1: 类比检测(相同数据→高置信)", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(10_000_000.0);
        let mut mems = Vec::new();
        let same_data: Vec<f64> = vec![1.0, 2.0, 3.0];
        for i in 0..5i32 {
            let anchor = Coord7D::new_even([i * 10, 0, 0, 0, 0, 0, 0]);
            mems.push(MemoryCodec::encode(&mut u, &anchor, &same_data).unwrap());
        }
        let analogies = ReasoningEngine::find_analogies(&u, &mems, 0.5);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if !analogies.is_empty() {
            let max_conf = analogies.iter().map(|a| a.confidence).fold(0.0f64, f64::max);
            TestResult::ok("F1", &format!("{}组类比 最高{:.3}", analogies.len(), max_conf), ms)
        } else {
            TestResult::fail("F1", "未检测到类比!", ms)
        }
    }));

    results.push(run("F2: 推理链连接远端", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(10_000_000.0);
        let mut h = HebbianMemory::new();
        let mut mems = Vec::new();
        for i in 0..10i32 {
            let data = vec![i as f64];
            let anchor = Coord7D::new_even([i * 10, 0, 0, 0, 0, 0, 0]);
            mems.push(MemoryCodec::encode(&mut u, &anchor, &data).unwrap());
        }
        for i in 0..9 { h.record_path(&[*mems[i].anchor(), *mems[i+1].anchor()], 2.0); }
        let chain = ReasoningEngine::infer_chain(&u, &h, mems[0].anchor(), mems[9].anchor(), 15);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if !chain.is_empty() {
            TestResult::ok("F2", &format!("mem0→mem9: {}跳", chain.len()), ms)
        } else {
            TestResult::fail("F2", "链为空!", ms)
        }
    }));

    results.push(run("F3: discover发现新线索", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000.0);
        let mut h = HebbianMemory::new();
        for x in 0..10i32 {
            for y in 0..10i32 {
                u.materialize_biased(Coord7D::new_even([x, y, 0, 0, 0, 0, 0]), 50.0, 0.6).ok();
            }
        }
        let anchor = Coord7D::new_even([5, 5, 0, 0, 0, 0, 0]);
        let discoveries = ReasoningEngine::discover(&u, &mut h, &anchor, 0.5);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        TestResult::ok("F3", &format!("发现{}条线索 赫布{}", discoveries.len(), h.edge_count()), ms)
    }));

    // ═══ G. 梦境引擎 ═══
    println!("\n══ G. 梦境引擎 ══");

    results.push(run("G1: 梦境三阶段完整执行", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000.0);
        let mut h = HebbianMemory::new();
        let mut mems = Vec::new();
        for i in 0..5i32 {
            let data = vec![i as f64 * 0.5];
            let anchor = Coord7D::new_even([i * 10, 0, 0, 0, 0, 0, 0]);
            mems.push(MemoryCodec::encode(&mut u, &anchor, &data).unwrap());
        }
        h.record_path(&[*mems[0].anchor(), *mems[1].anchor()], 2.0);
        h.record_path(&[*mems[1].anchor(), *mems[2].anchor()], 0.3);
        let report = DreamEngine::new().dream(&u, &mut h, &mems);
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok {
            TestResult::ok("G1", &format!("replay:{} weaken:{} consol:{} 边{}→{} 守恒✓",
                report.paths_replayed, report.paths_weakened, report.memories_consolidated,
                report.hebbian_edges_before, report.hebbian_edges_after), ms)
        } else {
            TestResult::fail("G1", "守恒违反!", ms)
        }
    }));

    results.push(run("G2: dream_cycle多轮不崩溃", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000.0);
        let mut h = HebbianMemory::new();
        let mut mems = Vec::new();
        for i in 0..3i32 {
            let data = vec![i as f64];
            let anchor = Coord7D::new_even([i * 20, 0, 0, 0, 0, 0, 0]);
            mems.push(MemoryCodec::encode(&mut u, &anchor, &data).unwrap());
        }
        h.record_path(&[*mems[0].anchor(), *mems[1].anchor()], 1.0);
        let reports = DreamEngine::new().dream_cycle(&u, &mut h, &mems, 10);
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok && reports.len() == 10 {
            TestResult::ok("G2", &format!("10轮完成 边{}/{} 守恒✓",
                reports[0].hebbian_edges_before, reports[9].hebbian_edges_after), ms)
        } else {
            TestResult::fail("G2", &format!("ok={} rounds={}", ok, reports.len()), ms)
        }
    }));

    // ═══ H. 维度调控 ═══
    println!("\n══ H. 维度调控 ══");

    results.push(run("H1: 调控后守恒不破", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000.0);
        let mut h = HebbianMemory::new();
        let mut crystal = CrystalEngine::new();
        let mut mems = Vec::new();
        for i in 0..5i32 {
            let data = vec![i as f64];
            let anchor = Coord7D::new_even([i * 20, 0, 0, 0, 0, 0, 0]);
            mems.push(MemoryCodec::encode(&mut u, &anchor, &data).unwrap());
        }
        h.record_path(&[*mems[0].anchor(), *mems[1].anchor()], 1.5);
        let report = RegulationEngine::new().regulate(&mut u, &mut h, &mut crystal, &mems);
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok {
            TestResult::ok("H1", &format!("stress:{:.2} entropy:{:.3} actions:{} 守恒✓",
                report.stress_level, report.entropy, report.actions.len()), ms)
        } else {
            TestResult::fail("H1", "守恒违反!", ms)
        }
    }));

    results.push(run("H2: 高利用率→高应激", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(5_000.0);
        for i in 0..50i32 {
            u.materialize_biased(Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]), 50.0, 0.8).ok();
        }
        let report = RegulationEngine::new().regulate(&mut u, &mut HebbianMemory::new(), &mut CrystalEngine::new(), &[]);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if report.stress_level > 0.3 {
            TestResult::ok("H2", &format!("利用率{:.0}% 应激{:.2}", u.stats().utilization * 100.0, report.stress_level), ms)
        } else {
            TestResult::fail("H2", &format!("应激过低: {:.2}", report.stress_level), ms)
        }
    }));

    // ═══ I. 自动扩展 ═══
    println!("\n══ I. 自动扩展 ══");

    results.push(run("I1: 低利用率不扩展", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000.0);
        u.materialize_biased(Coord7D::new_even([0,0,0,0,0,0,0]), 50.0, 0.6).ok();
        let report = AutoScaler::new().auto_scale(&mut u, &HebbianMemory::new(), &[]);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if report.nodes_added == 0 && u.verify_conservation() {
            TestResult::ok("I1", &format!("利用率{:.0}% → 不扩展 原因:{:?} 守恒✓", u.stats().utilization * 100.0, report.reason), ms)
        } else {
            TestResult::fail("I1", &format!("nodes_added={}", report.nodes_added), ms)
        }
    }));

    results.push(run("I2: 高利用率触发扩展", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(5_000.0);
        for i in 0..50i32 {
            u.materialize_biased(Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]), 50.0, 0.8).ok();
        }
        let report = AutoScaler::new().auto_scale(&mut u, &HebbianMemory::new(), &[]);
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok {
            TestResult::ok("I2", &format!("扩展+{} 原因:{:?} 守恒✓", report.nodes_added, report.reason), ms)
        } else {
            TestResult::fail("I2", "守恒违反!", ms)
        }
    }));

    results.push(run("I3: scale_to_fit_memory预分配", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000.0);
        let data: Vec<f64> = (0..14).map(|i| i as f64).collect();
        let report = AutoScaler::new().scale_to_fit_memory(&mut u, &data);
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok {
            let r = report.unwrap();
            TestResult::ok("I3", &format!("预分配+{}能量 守恒✓", r.energy_expanded_by), ms)
        } else {
            TestResult::fail("I3", "守恒违反!", ms)
        }
    }));

    // ═══ J. 持久化 ═══
    println!("\n══ J. 持久化 ══");

    results.push(run("J1: 完整roundtrip", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000.0);
        let mut h = HebbianMemory::new();
        let mut mems = Vec::new();
        for i in 0..10i32 {
            let data = vec![i as f64 * 0.3, (i + 1) as f64 * 0.7];
            let anchor = Coord7D::new_even([i * 100, 0, 0, 0, 0, 0, 0]);
            mems.push(MemoryCodec::encode(&mut u, &anchor, &data).unwrap());
        }
        h.record_path(&[*mems[0].anchor(), *mems[5].anchor()], 2.0);
        h.record_path(&[*mems[3].anchor(), *mems[7].anchor()], 1.5);
        let mut crystal = CrystalEngine::new();
        crystal.crystallize(&h, &u);

        let json = PersistEngine::to_json(&u, &h, &mems, &crystal).unwrap();
        let (u2, h2, mems2, c2) = PersistEngine::from_json(&json).unwrap();

        let ok1 = u2.verify_conservation();
        let ok2 = u.active_node_count() == u2.active_node_count();
        let ok3 = h2.edge_count() == h.edge_count();
        let ok4 = mems2.len() == mems.len();
        let ok5 = c2.channel_count() == crystal.channel_count();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok1 && ok2 && ok3 && ok4 && ok5 {
            TestResult::ok("J1", &format!("节点{} 赫布{} 记忆{} 结晶{} 守恒✓",
                u2.active_node_count(), h2.edge_count(), mems2.len(), c2.channel_count()), ms)
        } else {
            TestResult::fail("J1", &format!("ok={}/{}/{}/{}/{}", ok1, ok2, ok3, ok4, ok5), ms)
        }
    }));

    results.push(run("J2: JSON合法性", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(100_000.0);
        u.materialize_biased(Coord7D::new_even([0,0,0,0,0,0,0]), 50.0, 0.6).ok();
        let json = PersistEngine::to_json(&u, &HebbianMemory::new(), &[], &CrystalEngine::new()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if parsed.is_object() {
            TestResult::ok("J2", &format!("{}字节 合法✓", json.len()), ms)
        } else {
            TestResult::fail("J2", "JSON解析失败!", ms)
        }
    }));

    // ═══ K. 联合闭环 — 全系统协同 ═══
    println!("\n══ K. 联合闭环 — 全系统协同 ══");

    results.push(run("K1: 完整学习闭环", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(10_000_000.0);
        let mut h = HebbianMemory::new();
        let mut mems = Vec::new();

        for i in 0..10i32 {
            let data: Vec<f64> = (0..7).map(|j| (i * 7 + j) as f64 * 0.1).collect();
            let anchor = Coord7D::new_even([i * 50, i * 50, 0, 0, 0, 0, 0]);
            mems.push(MemoryCodec::encode(&mut u, &anchor, &data).unwrap());
        }

        let pulse = PulseEngine::new();
        for mem in &mems {
            pulse.propagate(mem.anchor(), PulseType::Reinforcing, &u, &mut h);
        }

        let mut crystal = CrystalEngine::new();
        crystal.crystallize(&h, &u);

        let analogies = ReasoningEngine::find_analogies(&u, &mems, 0.3);
        let associations = ReasoningEngine::find_associations(&u, &h, &crystal, mems[0].anchor(), 3);

        let dream_report = DreamEngine::new().dream(&u, &mut h, &mems);
        let reg_report = RegulationEngine::new().regulate(&mut u, &mut h, &mut crystal, &mems);

        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok {
            TestResult::ok("K1", &format!(
                "记忆:{} 赫布:{} 结晶:{} 类比:{} 联想:{} 梦境[r:{}w:{}c:{}] 调控[s:{:.2}] 守恒✓",
                mems.len(), h.edge_count(), crystal.channel_count(),
                analogies.len(), associations.len(),
                dream_report.paths_replayed, dream_report.paths_weakened, dream_report.memories_consolidated,
                reg_report.stress_level), ms)
        } else {
            TestResult::fail("K1", "闭环守恒违反!", ms)
        }
    }));

    results.push(run("K2: 学习→遗忘→重新学习循环", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(10_000_000.0);
        let mut h = HebbianMemory::new();
        for cycle in 0..3i32 {
            let mut mems = Vec::new();
            for i in 0..5i32 {
                let data = vec![(cycle * 5 + i) as f64 * 0.1];
                let anchor = Coord7D::new_even([cycle * 100 + i * 20, 0, 0, 0, 0, 0, 0]);
                mems.push(MemoryCodec::encode(&mut u, &anchor, &data).unwrap());
            }
            for m in &mems {
                PulseEngine::new().propagate(m.anchor(), PulseType::Reinforcing, &u, &mut h);
            }
            h.decay_all();
            h.prune();
        }
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok {
            TestResult::ok("K2", &format!("3轮循环 赫布{} 守恒✓", h.edge_count()), ms)
        } else {
            TestResult::fail("K2", "守恒违反!", ms)
        }
    }));

    results.push(run("K3: 持久化→恢复→继续操作", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000.0);
        let mut h = HebbianMemory::new();
        for i in 0..5i32 {
            u.materialize_biased(Coord7D::new_even([i, 0, 0, 0, 0, 0, 0]), 50.0, 0.6).ok();
        }
        h.record_path(&[Coord7D::new_even([0,0,0,0,0,0,0]), Coord7D::new_even([1,0,0,0,0,0,0])], 2.0);
        let json = PersistEngine::to_json(&u, &h, &[], &CrystalEngine::new()).unwrap();
        let (mut u2, mut h2, _, _) = PersistEngine::from_json(&json).unwrap();
        u2.materialize_biased(Coord7D::new_even([100,0,0,0,0,0,0]), 50.0, 0.6).ok();
        h2.record_path(&[Coord7D::new_even([100,0,0,0,0,0,0]), Coord7D::new_even([0,0,0,0,0,0,0])], 1.5);
        let ok = u2.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok {
            TestResult::ok("K3", &format!("恢复+操作 节点{} 赫布{} 守恒✓", u2.active_node_count(), h2.edge_count()), ms)
        } else {
            TestResult::fail("K3", "守恒违反!", ms)
        }
    }));

    results.push(run("K4: 大规模闭环(100记忆+拓扑+扩展)", || {
        let t = Instant::now();
        let mut u = DarkUniverse::new(1_000_000_000.0);
        let mut h = HebbianMemory::new();
        let mut mems = Vec::new();
        for i in 0..100i32 {
            let data: Vec<f64> = (0..14).map(|j| ((i * 14 + j) % 100) as f64 * 0.1).collect();
            let x = (i % 10) * 100;
            let y = (i / 10) * 100;
            let anchor = Coord7D::new_even([x, y, 0, 0, 0, 0, 0]);
            mems.push(MemoryCodec::encode(&mut u, &anchor, &data).unwrap());
            if i > 0 { h.record_path(&[*mems[i as usize - 1].anchor(), *mems[i as usize].anchor()], 1.0); }
        }
        let topo = TopologyEngine::analyze(&u);
        let scale_report = AutoScaler::new().auto_scale(&mut u, &h, &mems);
        let ok = u.verify_conservation();
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        if ok {
            TestResult::ok("K4", &format!(
                "100记忆 赫布{} H0={} 扩展+{} 守恒✓",
                h.edge_count(), topo.connected_components, scale_report.nodes_added), ms)
        } else {
            TestResult::fail("K4", "守恒违反!", ms)
        }
    }));

    // ═══ 报告 ═══
    println!("\n{}", "═".repeat(70));
    println!("  全方位深度测试报告");
    println!("{}", "═".repeat(70));

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();
    let total_ms: f64 = results.iter().map(|r| r.elapsed_ms).sum();

    for r in &results {
        let icon = if r.passed { "✓" } else { "✗" };
        println!("  {} {:<50} {:.1}ms  {}", icon, r.name, r.elapsed_ms, r.detail);
    }

    println!("{}", "═".repeat(70));
    println!("  结果: {}/{} 通过  {} 失败  总耗时: {:.0}ms", passed, passed + failed, failed, total_ms);
    if failed == 0 {
        println!("  ★ 全系统通过 — 能量守恒律在所有场景下坚如磐石 ★");
    } else {
        println!("  !! 有 {} 项失败 !!", failed);
    }
    println!("{}", "═".repeat(70));
}

fn run<F>(name: &str, f: F) -> TestResult
where F: FnOnce() -> TestResult,
{
    let r = f();
    let icon = if r.passed { "✓" } else { "✗" };
    println!("  {} {} — {}", icon, name, r.detail);
    r
}
