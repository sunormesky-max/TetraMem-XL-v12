use lazy_static::lazy_static;
use prometheus::{
    self, Encoder, Histogram, HistogramOpts, IntCounter, IntGauge, Opts, TextEncoder,
};

lazy_static! {
    pub static ref NODES_TOTAL: IntGauge =
        IntGauge::new("tetramem_nodes_total", "Total number of nodes in universe").unwrap();
    pub static ref NODES_MANIFESTED: IntGauge =
        IntGauge::new("tetramem_nodes_manifested", "Number of manifested nodes").unwrap();
    pub static ref NODES_DARK: IntGauge =
        IntGauge::new("tetramem_nodes_dark", "Number of dark nodes").unwrap();
    pub static ref ENERGY_TOTAL: IntGauge = IntGauge::with_opts(
        Opts::new("energy_total", "Total energy in universe").namespace("tetramem")
    )
    .unwrap();
    pub static ref ENERGY_ALLOCATED: IntGauge = IntGauge::with_opts(
        Opts::new("energy_allocated", "Allocated energy").namespace("tetramem")
    )
    .unwrap();
    pub static ref ENERGY_AVAILABLE: IntGauge = IntGauge::with_opts(
        Opts::new("energy_available", "Available energy").namespace("tetramem")
    )
    .unwrap();
    pub static ref MEMORIES_TOTAL: IntGauge =
        IntGauge::new("tetramem_memories_total", "Total stored memories").unwrap();
    pub static ref HEBBIAN_EDGES: IntGauge =
        IntGauge::new("tetramem_hebbian_edges", "Total Hebbian edges").unwrap();
    pub static ref API_REQUESTS_TOTAL: IntCounter =
        IntCounter::new("tetramem_api_requests_total", "Total API requests").unwrap();
    pub static ref API_ENCODE_TOTAL: IntCounter = IntCounter::new(
        "tetramem_api_encode_total",
        "Total memory encode operations"
    )
    .unwrap();
    pub static ref API_DECODE_TOTAL: IntCounter = IntCounter::new(
        "tetramem_api_decode_total",
        "Total memory decode operations"
    )
    .unwrap();
    pub static ref API_PULSE_TOTAL: IntCounter =
        IntCounter::new("tetramem_api_pulse_total", "Total pulse operations").unwrap();
    pub static ref API_DREAM_TOTAL: IntCounter =
        IntCounter::new("tetramem_api_dream_total", "Total dream cycle operations").unwrap();
    pub static ref REQUEST_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new("request_duration_seconds", "Request duration")
            .namespace("tetramem")
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
    )
    .unwrap();
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
    let registry = prometheus::default_registry();
    registry.register(Box::new(NODES_TOTAL.clone())).unwrap();
    registry
        .register(Box::new(NODES_MANIFESTED.clone()))
        .unwrap();
    registry.register(Box::new(NODES_DARK.clone())).unwrap();
    registry.register(Box::new(ENERGY_TOTAL.clone())).unwrap();
    registry
        .register(Box::new(ENERGY_ALLOCATED.clone()))
        .unwrap();
    registry
        .register(Box::new(ENERGY_AVAILABLE.clone()))
        .unwrap();
    registry.register(Box::new(MEMORIES_TOTAL.clone())).unwrap();
    registry.register(Box::new(HEBBIAN_EDGES.clone())).unwrap();
    registry
        .register(Box::new(API_REQUESTS_TOTAL.clone()))
        .unwrap();
    registry
        .register(Box::new(API_ENCODE_TOTAL.clone()))
        .unwrap();
    registry
        .register(Box::new(API_DECODE_TOTAL.clone()))
        .unwrap();
    registry
        .register(Box::new(API_PULSE_TOTAL.clone()))
        .unwrap();
    registry
        .register(Box::new(API_DREAM_TOTAL.clone()))
        .unwrap();
    registry
        .register(Box::new(REQUEST_DURATION.clone()))
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
    NODES_TOTAL.set(nodes as i64);
    NODES_MANIFESTED.set(manifested as i64);
    NODES_DARK.set(dark as i64);
    ENERGY_TOTAL.set((total_energy + 0.5) as i64);
    ENERGY_ALLOCATED.set((allocated + 0.5) as i64);
    ENERGY_AVAILABLE.set((available + 0.5) as i64);
    MEMORIES_TOTAL.set(memories as i64);
    HEBBIAN_EDGES.set(hebbian_edges as i64);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_init_and_render() {
        init_metrics();
        NODES_TOTAL.set(99999);
        let output = render_metrics();
        assert!(output.contains("tetramem_nodes_total"));
        assert!(output.contains("99999"));
    }

    #[test]
    fn update_universe_metrics_works() {
        init_metrics();
        update_universe_metrics(100, 60, 40, 10000.0, 8000.0, 2000.0, 50, 200);
        assert_eq!(NODES_TOTAL.get(), 100);
        assert_eq!(NODES_MANIFESTED.get(), 60);
        assert_eq!(MEMORIES_TOTAL.get(), 50);
    }
}
