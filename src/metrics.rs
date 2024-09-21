use axum_prometheus::metrics::{Counter, Gauge, Histogram};

pub fn metric_counter(name: &'static str) -> Counter {
    axum_prometheus::metrics::counter!(name)
}

pub fn metric_gauge(name: &'static str) -> Gauge {
    axum_prometheus::metrics::gauge!(name)
}

pub fn metric_histogram(name: &'static str) -> Histogram {
    axum_prometheus::metrics::histogram!(name)
}
