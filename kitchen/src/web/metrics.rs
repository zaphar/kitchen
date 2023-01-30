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
//! A [metrics] powered [TraceLayer] that works with any [Tower](https://crates.io/crates/tower) middleware.
use axum::http::{Request, Response};
use metrics::{histogram, increment_counter, Label};
use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::{
        DefaultMakeSpan, DefaultOnEos, OnBodyChunk, OnFailure, OnRequest, OnResponse, TraceLayer,
    },
};
use tracing;

/// A Metrics Trace Layer using a [MetricsRecorder].
///
/// The layer will record 4 different metrics:
///
/// * http_request_counter
/// * http_request_failure_counter
/// * http_request_size_bytes_hist
/// * http_request_request_time_micros_hist
///
/// Each of the metrics are labled by host, method, and path
pub type MetricsTraceLayer<B, F> = TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    DefaultMakeSpan,
    MetricsRecorder<B, F>,
    MetricsRecorder<B, F>,
    MetricsRecorder<B, F>,
    DefaultOnEos,
    MetricsRecorder<B, F>,
>;

/// Holds the state required for recording metrics on a given request.
pub struct MetricsRecorder<B, F>
where
    F: Fn(&B) -> u64,
{
    labels: Arc<Mutex<Vec<Label>>>,
    size: Arc<AtomicU64>,
    chunk_len: Arc<F>,
    _phantom: PhantomData<B>,
}

impl<B, F> Clone for MetricsRecorder<B, F>
where
    F: Fn(&B) -> u64,
{
    fn clone(&self) -> Self {
        Self {
            labels: self.labels.clone(),
            size: self.size.clone(),
            chunk_len: self.chunk_len.clone(),
            _phantom: self._phantom.clone(),
        }
    }
}

impl<B, F> MetricsRecorder<B, F>
where
    F: Fn(&B) -> u64,
{
    /// Construct a new [MetricsRecorder] using the installed [Recorder].
    pub fn new(f: F) -> Self {
        Self {
            labels: Arc::new(Mutex::new(Vec::new())),
            size: Arc::new(AtomicU64::new(0)),
            chunk_len: Arc::new(f),
            _phantom: PhantomData,
        }
    }
}

impl<B, F> OnBodyChunk<B> for MetricsRecorder<B, F>
where
    F: Fn(&B) -> u64,
{
    fn on_body_chunk(&mut self, chunk: &B, _latency: std::time::Duration, _span: &tracing::Span) {
        let _ = self
            .size
            .fetch_add(self.chunk_len.as_ref()(chunk), Ordering::SeqCst);
    }
}

impl<B, FailureClass, F> OnFailure<FailureClass> for MetricsRecorder<B, F>
where
    F: Fn(&B) -> u64,
{
    fn on_failure(
        &mut self,
        _failure_classification: FailureClass,
        _latency: std::time::Duration,
        _span: &tracing::Span,
    ) {
        let labels = self.labels.lock().expect("Failed to unlock labels").clone();
        increment_counter!("http_request_failure_counter", labels);
    }
}

impl<B, RB, F> OnResponse<RB> for MetricsRecorder<B, F>
where
    F: Fn(&B) -> u64,
{
    fn on_response(
        self,
        _response: &Response<RB>,
        latency: std::time::Duration,
        _span: &tracing::Span,
    ) {
        let labels = self.labels.lock().expect("Failed to unlock labels").clone();
        histogram!(
            "http_request_time_micros_hist",
            latency.as_micros() as f64,
            labels.clone()
        );
        histogram!(
            "http_request_size_bytes_hist",
            self.size.as_ref().load(Ordering::SeqCst) as f64,
            labels
        )
    }
}

fn make_request_lables(path: String, host: String, method: String) -> Vec<Label> {
    vec![
        Label::new("path", path),
        Label::new("host", host),
        Label::new("method", method),
    ]
}

impl<B, RB, F> OnRequest<RB> for MetricsRecorder<B, F>
where
    F: Fn(&B) -> u64,
{
    fn on_request(&mut self, request: &Request<RB>, _span: &tracing::Span) {
        let path = request.uri().path().to_lowercase();
        let host = request.uri().host().unwrap_or("").to_lowercase();
        let method = request.method().to_string();

        let labels = make_request_lables(path, host, method);
        let mut labels_lock = self.labels.lock().expect("Failed to unlock labels");
        (*labels_lock.as_mut()) = labels.clone();
        increment_counter!("http_request_counter", labels);
    }
}

/// Construct a [TraceLayer] that will use an installed [metrics::Recorder] to record metrics per request.
pub fn make_layer<B, F>(f: F) -> MetricsTraceLayer<B, F>
where
    F: Fn(&B) -> u64,
{
    let metrics_recorder = MetricsRecorder::new(f);
    let layer = TraceLayer::new_for_http()
        .on_body_chunk(metrics_recorder.clone())
        .on_request(metrics_recorder.clone())
        .on_response(metrics_recorder.clone())
        .on_failure(metrics_recorder.clone());
    layer
}
