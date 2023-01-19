// Copyright 2023 Jeremy Wall (Jeremy@marzhilsltudios.com)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

use axum::{body::Bytes, http::Request, http::Response};
use metrics::{Key, Label, Recorder};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusRecorder};
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::{
        DefaultMakeSpan, DefaultOnEos, OnBodyChunk, OnEos, OnFailure, OnRequest, OnResponse,
        TraceLayer,
    },
};

// We want to track requeste count, request latency, request size minimally.

pub type MetricsTraceLayer = TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    DefaultMakeSpan,
    MetricsRecorder,
    MetricsRecorder,
    MetricsRecorder,
    DefaultOnEos,
    MetricsRecorder,
>;

pub fn get_recorder() -> PrometheusRecorder {
    let builder = PrometheusBuilder::new();
    builder.build_recorder()
}

#[derive(Clone)]
pub struct MetricsRecorder {
    rec: Arc<PrometheusRecorder>,
    labels: Arc<Mutex<Vec<Label>>>,
    size: Arc<AtomicU64>,
}

impl MetricsRecorder {
    pub fn new_with_rec(rec: Arc<PrometheusRecorder>) -> Self {
        Self {
            rec,
            labels: Arc::new(Mutex::new(Vec::new())),
            size: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl OnBodyChunk<Bytes> for MetricsRecorder {
    fn on_body_chunk(
        &mut self,
        chunk: &Bytes,
        _latency: std::time::Duration,
        _span: &tracing::Span,
    ) {
        let _ = self.size.fetch_add(chunk.len() as u64, Ordering::SeqCst);
    }
}

impl OnEos for MetricsRecorder {
    fn on_eos(
        self,
        _trailers: Option<&axum::http::HeaderMap>,
        _stream_duration: std::time::Duration,
        _span: &tracing::Span,
    ) {
    }
}

impl<FailureClass> OnFailure<FailureClass> for MetricsRecorder {
    fn on_failure(
        &mut self,
        _failure_classification: FailureClass,
        _latency: std::time::Duration,
        _span: &tracing::Span,
    ) {
        let labels = self.labels.lock().expect("Failed to unlock labels").clone();
        self.rec
            .as_ref()
            .register_histogram(&Key::from_parts("http_request_failure_counter", labels));
    }
}

impl<B> OnResponse<B> for MetricsRecorder {
    fn on_response(
        self,
        _response: &Response<B>,
        latency: std::time::Duration,
        _span: &tracing::Span,
    ) {
        let labels = self.labels.lock().expect("Failed to unlock labels").clone();
        self.rec
            .as_ref()
            .register_histogram(&Key::from_parts("http_request_time_ms", labels.clone()))
            // If we somehow end up having requests overflow from u128 into f64 then we have
            // much bigger problems than this cast.
            .record(latency.as_micros() as f64);
        self.rec
            .as_ref()
            .register_histogram(&Key::from_parts("http_request_size_bytes", labels))
            .record(self.size.as_ref().load(Ordering::SeqCst) as f64);
    }
}

fn make_request_lables(path: String, host: String, method: String) -> Vec<Label> {
    vec![
        Label::new("path", path),
        Label::new("host", host),
        Label::new("method", method),
    ]
}

impl<B> OnRequest<B> for MetricsRecorder {
    fn on_request(&mut self, request: &Request<B>, _span: &tracing::Span) {
        let rec = self.rec.as_ref();
        let path = request.uri().path().to_lowercase();
        let host = request.uri().host().unwrap_or("").to_lowercase();
        let method = request.method().to_string();

        let labels = make_request_lables(path, host, method);
        let mut labels_lock = self.labels.lock().expect("Failed to unlock labels");
        (*labels_lock.as_mut()) = labels.clone();
        rec.register_counter(&Key::from_parts("http_request_counter", labels.clone()))
            .increment(1);
    }
}

pub fn make_trace_layer(rec: Arc<PrometheusRecorder>) -> MetricsTraceLayer {
    let metrics_recorder = MetricsRecorder::new_with_rec(rec);
    let layer = TraceLayer::new_for_http()
        .on_body_chunk(metrics_recorder.clone())
        .on_request(metrics_recorder.clone())
        .on_response(metrics_recorder.clone())
        .on_failure(metrics_recorder.clone());
    layer
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_construction() {
        let metrics_recorder = MetricsRecorder::new_with_rec(std::sync::Arc::new(get_recorder()));
        let _trace_layer = TraceLayer::new_for_http()
            .on_body_chunk(metrics_recorder.clone())
            .on_request(metrics_recorder.clone())
            .on_response(metrics_recorder.clone())
            .on_failure(metrics_recorder.clone());
    }
}
