use axum_prometheus::metrics::{Counter, Gauge, Histogram};

/// Gets a prometheus counter
pub fn metric_counter(name: &'static str) -> Counter {
    axum_prometheus::metrics::counter!(name)
}

/// Gets a prometheus gauge
pub fn metric_gauge(name: &'static str) -> Gauge {
    axum_prometheus::metrics::gauge!(name)
}

/// Gets a prometheus histogram
pub fn metric_histogram(name: &'static str) -> Histogram {
    axum_prometheus::metrics::histogram!(name)
}
