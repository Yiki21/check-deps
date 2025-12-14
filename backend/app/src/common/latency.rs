use std::{fmt::Display, time::Duration};

use tower_http::trace::OnResponse;

#[derive(Clone)]
pub struct LatencyResponse;

impl<T> OnResponse<T> for LatencyResponse {
    fn on_response(
        self,
        response: &axum::http::Response<T>,
        latency: std::time::Duration,
        _: &tracing::Span,
    ) {
        tracing::info!(
            latency = %Latency(latency),
            status = %response.status(),
            "finished processing request"
        );
    }
}

struct Latency(Duration);

impl Display for Latency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.as_millis() > 0 {
            write!(f, "{} ms", self.0.as_millis())
        } else {
            write!(f, "{} µs", self.0.as_micros())
        }
    }
}
