// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025 sunormesky-max (Liu Qihang)
// TetraMem-XL v12.0 — 7D Dark Universe Memory System
use prometheus::{
    self, Encoder, Histogram, HistogramOpts, IntCounter, IntGauge, Opts, TextEncoder,
};

pub static NODES_TOTAL: std::sync::OnceLock<IntGauge> = std::sync::OnceLock::new();
pub static NODES_MANIFESTED: std::sync::OnceLock<IntGauge> = std::sync::OnceLock::new();
pub static NODES_DARK: std::sync::OnceLock<IntGauge> = std::sync::OnceLock::new();
pub static ENERGY_TOTAL: std::sync::OnceLock<IntGauge> = std::sync::OnceLock::new();
pub static ENERGY_ALLOCATED: std::sync::OnceLock<IntGauge> = std::sync::OnceLock::new();
pub static ENERGY_AVAILABLE: std::sync::OnceLock<IntGauge> = std::sync::OnceLock::new();
pub static MEMORIES_TOTAL: std::sync::OnceLock<IntGauge> = std::sync::OnceLock::new();
pub static HEBBIAN_EDGES: std::sync::OnceLock<IntGauge> = std::sync::OnceLock::new();
pub static API_REQUESTS_TOTAL: std::sync::OnceLock<IntCounter> = std::sync::OnceLock::new();
pub static API_ENCODE_TOTAL: std::sync::OnceLock<IntCounter> = std::sync::OnceLock::new();
pub static API_DECODE_TOTAL: std::sync::OnceLock<IntCounter> = std::sync::OnceLock::new();
pub static API_PULSE_TOTAL: std::sync::OnceLock<IntCounter> = std::sync::OnceLock::new();
pub static API_DREAM_TOTAL: std::sync::OnceLock<IntCounter> = std::sync::OnceLock::new();
pub static REQUEST_DURATION: std::sync::OnceLock<Histogram> = std::sync::OnceLock::new();

fn init_or_get_metrics() -> &'static MetricsRefs {
    static METRICS: std::sync::OnceLock<MetricsRefs> = std::sync::OnceLock::new();
    METRICS.get_or_init(|| {
        let nt =
            IntGauge::new("tetramem_nodes_total", "Total number of nodes in universe").unwrap();
        let nm = IntGauge::new("tetramem_nodes_manifested", "Number of manifested nodes").unwrap();
        let nd = IntGauge::new("tetramem_nodes_dark", "Number of dark nodes").unwrap();
        let et = IntGauge::with_opts(
            Opts::new("energy_total", "Total energy in universe").namespace("tetramem"),
        )
        .unwrap();
        let ea = IntGauge::with_opts(
            Opts::new("energy_allocated", "Allocated energy").namespace("tetramem"),
        )
        .unwrap();
        let ev = IntGauge::with_opts(
            Opts::new("energy_available", "Available energy").namespace("tetramem"),
        )
        .unwrap();
        let mt = IntGauge::new("tetramem_memories_total", "Total stored memories").unwrap();
        let he = IntGauge::new("tetramem_hebbian_edges", "Total Hebbian edges").unwrap();
        let art = IntCounter::new("tetramem_api_requests_total", "Total API requests").unwrap();
        let aet = IntCounter::new(
            "tetramem_api_encode_total",
            "Total memory encode operations",
        )
        .unwrap();
        let adt = IntCounter::new(
            "tetramem_api_decode_total",
            "Total memory decode operations",
        )
        .unwrap();
        let apt = IntCounter::new("tetramem_api_pulse_total", "Total pulse operations").unwrap();
        let adrm =
            IntCounter::new("tetramem_api_dream_total", "Total dream cycle operations").unwrap();
        let rd = Histogram::with_opts(
            HistogramOpts::new("request_duration_seconds", "Request duration")
                .namespace("tetramem")
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]),
        )
        .unwrap();

        NODES_TOTAL.get_or_init(|| nt.clone());
        NODES_MANIFESTED.get_or_init(|| nm.clone());
        NODES_DARK.get_or_init(|| nd.clone());
        ENERGY_TOTAL.get_or_init(|| et.clone());
        ENERGY_ALLOCATED.get_or_init(|| ea.clone());
        ENERGY_AVAILABLE.get_or_init(|| ev.clone());
        MEMORIES_TOTAL.get_or_init(|| mt.clone());
        HEBBIAN_EDGES.get_or_init(|| he.clone());
        API_REQUESTS_TOTAL.get_or_init(|| art.clone());
        API_ENCODE_TOTAL.get_or_init(|| aet.clone());
        API_DECODE_TOTAL.get_or_init(|| adt.clone());
        API_PULSE_TOTAL.get_or_init(|| apt.clone());
        API_DREAM_TOTAL.get_or_init(|| adrm.clone());
        REQUEST_DURATION.get_or_init(|| rd.clone());

        MetricsRefs {
            nodes_total: nt,
            nodes_manifested: nm,
            nodes_dark: nd,
            energy_total: et,
            energy_allocated: ea,
            energy_available: ev,
            memories_total: mt,
            hebbian_edges: he,
            api_requests_total: art,
            api_encode_total: aet,
            api_decode_total: adt,
            api_pulse_total: apt,
            api_dream_total: adrm,
            request_duration: rd,
        }
    })
}

struct MetricsRefs {
    nodes_total: IntGauge,
    nodes_manifested: IntGauge,
    nodes_dark: IntGauge,
    energy_total: IntGauge,
    energy_allocated: IntGauge,
    energy_available: IntGauge,
    memories_total: IntGauge,
    hebbian_edges: IntGauge,
    api_requests_total: IntCounter,
    api_encode_total: IntCounter,
    api_decode_total: IntCounter,
    api_pulse_total: IntCounter,
    api_dream_total: IntCounter,
    request_duration: Histogram,
}

use std::sync::atomic::{AtomicBool, Ordering};

static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn init_metrics() {
    if INITIALIZED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }
    let refs = init_or_get_metrics();
    let registry = prometheus::default_registry();
    registry
        .register(Box::new(refs.nodes_total.clone()))
        .unwrap();
    registry
        .register(Box::new(refs.nodes_manifested.clone()))
        .unwrap();
    registry
        .register(Box::new(refs.nodes_dark.clone()))
        .unwrap();
    registry
        .register(Box::new(refs.energy_total.clone()))
        .unwrap();
    registry
        .register(Box::new(refs.energy_allocated.clone()))
        .unwrap();
    registry
        .register(Box::new(refs.energy_available.clone()))
        .unwrap();
    registry
        .register(Box::new(refs.memories_total.clone()))
        .unwrap();
    registry
        .register(Box::new(refs.hebbian_edges.clone()))
        .unwrap();
    registry
        .register(Box::new(refs.api_requests_total.clone()))
        .unwrap();
    registry
        .register(Box::new(refs.api_encode_total.clone()))
        .unwrap();
    registry
        .register(Box::new(refs.api_decode_total.clone()))
        .unwrap();
    registry
        .register(Box::new(refs.api_pulse_total.clone()))
        .unwrap();
    registry
        .register(Box::new(refs.api_dream_total.clone()))
        .unwrap();
    registry
        .register(Box::new(refs.request_duration.clone()))
        .unwrap();
}

pub fn render_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap_or_default()
}

#[allow(clippy::too_many_arguments)]
pub fn update_universe_metrics(
    nodes: usize,
    manifested: usize,
    dark: usize,
    total_energy: f64,
    allocated: f64,
    available: f64,
    memories: usize,
    hebbian_edges: usize,
) {
    init_or_get_metrics();
    if let Some(g) = NODES_TOTAL.get() {
        g.set(nodes as i64);
    }
    if let Some(g) = NODES_MANIFESTED.get() {
        g.set(manifested as i64);
    }
    if let Some(g) = NODES_DARK.get() {
        g.set(dark as i64);
    }
    if let Some(g) = ENERGY_TOTAL.get() {
        g.set((total_energy + 0.5) as i64);
    }
    if let Some(g) = ENERGY_ALLOCATED.get() {
        g.set((allocated + 0.5) as i64);
    }
    if let Some(g) = ENERGY_AVAILABLE.get() {
        g.set((available + 0.5) as i64);
    }
    if let Some(g) = MEMORIES_TOTAL.get() {
        g.set(memories as i64);
    }
    if let Some(g) = HEBBIAN_EDGES.get() {
        g.set(hebbian_edges as i64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_init_and_render() {
        init_metrics();
        if let Some(g) = NODES_TOTAL.get() {
            g.set(99999);
        }
        let output = render_metrics();
        assert!(output.contains("tetramem_nodes_total"));
        assert!(output.contains("99999"));
    }

    #[test]
    fn update_universe_metrics_works() {
        init_metrics();
        update_universe_metrics(100, 60, 40, 10000.0, 8000.0, 2000.0, 50, 200);
        if let Some(g) = NODES_TOTAL.get() {
            assert_eq!(g.get(), 100);
        }
        if let Some(g) = NODES_MANIFESTED.get() {
            assert_eq!(g.get(), 60);
        }
        if let Some(g) = MEMORIES_TOTAL.get() {
            assert_eq!(g.get(), 50);
        }
    }
}
