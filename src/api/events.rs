use anyhow::{Context, Result};
use futures::stream::{Stream, StreamExt};
use reqwest::Client;
use std::pin::Pin;
use std::task::Poll;
use std::time::Duration;

use crate::types::Event;

/// Default base delay before attempting to reconnect after an SSE disconnect.
const RECONNECT_BASE_DELAY: Duration = Duration::from_secs(1);

/// Maximum number of consecutive reconnection attempts before the stream ends.
const MAX_RECONNECT_ATTEMPTS: u32 = 30;

// ===========================================================================
// EventStream  (instance-scoped: GET /event)
// ===========================================================================

/// A stream of SSE events from the OpenCode `/event` endpoint.
///
/// Automatically reconnects on disconnect with exponential back-off (capped at
/// 30 s) up to [`MAX_RECONNECT_ATTEMPTS`] consecutive failures.
pub struct EventStream {
    inner: Pin<Box<dyn Stream<Item = Result<Event>> + Send>>,
}

impl EventStream {
    /// Connect to the instance-scoped SSE event stream at `GET /event`.
    pub async fn connect(base_url: &str) -> Result<Self> {
        let url = format!("{}/event", base_url.trim_end_matches('/'));
        let stream = reconnecting_sse_stream(url, false);
        Ok(Self {
            inner: Box::pin(stream),
        })
    }

    /// Consume the next event from the stream.
    pub async fn next(&mut self) -> Option<Result<Event>> {
        self.inner.next().await
    }
}

impl Stream for EventStream {
    type Item = Result<Event>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

// ===========================================================================
// Reconnecting SSE stream
// ===========================================================================

/// Internal state carried across reconnection attempts.
struct ReconnectState {
    url: String,
    is_global: bool,
    attempts: u32,
    inner: Option<Pin<Box<dyn Stream<Item = Result<Event>> + Send>>>,
}

/// Build a stream that automatically reconnects to the SSE endpoint on
/// failure, with exponential back-off capped at 30 seconds.
fn reconnecting_sse_stream(
    url: String,
    is_global: bool,
) -> impl Stream<Item = Result<Event>> + Send {
    futures::stream::unfold(
        ReconnectState {
            url,
            is_global,
            attempts: 0,
            inner: None,
        },
        |mut state| async move {
            loop {
                // If we have an active inner stream, try to get the next item.
                if let Some(ref mut inner) = state.inner {
                    match inner.next().await {
                        Some(Ok(event)) => {
                            state.attempts = 0; // success resets the counter
                            return Some((Ok(event), state));
                        }
                        Some(Err(e)) => {
                            tracing::warn!(
                                "SSE stream error, will reconnect: {}",
                                e
                            );
                            state.inner = None;
                            // fall through to reconnect
                        }
                        None => {
                            tracing::info!("SSE stream ended, will reconnect");
                            state.inner = None;
                            // fall through to reconnect
                        }
                    }
                }

                // Give up after too many consecutive failures.
                if state.attempts >= MAX_RECONNECT_ATTEMPTS {
                    tracing::error!(
                        "SSE: giving up after {} reconnect attempts for {}",
                        MAX_RECONNECT_ATTEMPTS,
                        state.url
                    );
                    return None;
                }

                // Exponential back-off (skip delay on the very first connect).
                if state.attempts > 0 {
                    let exp = (state.attempts - 1).min(5); // cap exponent
                    let delay = RECONNECT_BASE_DELAY
                        .mul_f64(2.0_f64.powi(exp as i32))
                        .min(Duration::from_secs(30));
                    tracing::debug!(
                        "SSE: reconnecting to {} in {:?} (attempt {})",
                        state.url,
                        delay,
                        state.attempts + 1
                    );
                    tokio::time::sleep(delay).await;
                }
                state.attempts += 1;

                match connect_sse(&state.url, state.is_global).await {
                    Ok(stream) => {
                        tracing::debug!(
                            "SSE: connected to {} (attempt {})",
                            state.url,
                            state.attempts
                        );
                        state.inner = Some(stream);
                        // loop back to poll the new stream
                    }
                    Err(e) => {
                        tracing::warn!(
                            "SSE: connect to {} failed (attempt {}): {}",
                            state.url,
                            state.attempts,
                            e
                        );
                        // loop back to retry
                    }
                }
            }
        },
    )
}

// ===========================================================================
// Single-connection helpers
// ===========================================================================

/// Open one SSE connection and return a stream of parsed [`Event`] values.
async fn connect_sse(
    url: &str,
    is_global: bool,
) -> Result<Pin<Box<dyn Stream<Item = Result<Event>> + Send>>> {
    let client = Client::new();
    let response = client
        .get(url)
        .header("Accept", "text/event-stream")
        .send()
        .await
        .context("Failed to connect to SSE endpoint")?;

    if !response.status().is_success() {
        anyhow::bail!(
            "SSE connection to {} failed with status {}",
            url,
            response.status()
        );
    }

    let byte_stream = response.bytes_stream();
    let event_stream = sse_decode(byte_stream, is_global);
    Ok(Box::pin(event_stream))
}

// ===========================================================================
// SSE frame parsing
// ===========================================================================

/// Decode a raw byte stream into parsed SSE [`Event`] items.
///
/// SSE wire format:
/// ```text
/// event: <event-type>\n
/// data: <json>\n
/// \n
/// ```
///
/// When `is_global` is true the data JSON is expected to match:
/// `{ "directory": "...", "payload": <Event> }` -- we extract the `payload`.
fn sse_decode<S>(
    byte_stream: S,
    is_global: bool,
) -> impl Stream<Item = Result<Event>> + Send
where
    S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
{
    let line_stream: Pin<Box<dyn Stream<Item = Result<String, reqwest::Error>> + Send>> =
        Box::pin(byte_stream_to_lines(byte_stream));

    futures::stream::unfold(
        (line_stream, String::new(), String::new()),
        move |(mut stream, mut event_type, mut data_buf)| async move {
            loop {
                match stream.next().await {
                    Some(Ok(line)) => {
                        if line.is_empty() {
                            // Empty line signals end of one SSE frame.
                            if !data_buf.is_empty() {
                                let data = std::mem::take(&mut data_buf);
                                let _etype =
                                    std::mem::take(&mut event_type);

                                let parse_result = if is_global {
                                    parse_global_event_data(&data)
                                } else {
                                    serde_json::from_str::<Event>(&data)
                                        .map_err(Into::into)
                                };

                                match parse_result {
                                    Ok(event) => {
                                        return Some((
                                            Ok(event),
                                            (
                                                stream,
                                                event_type,
                                                data_buf,
                                            ),
                                        ));
                                    }
                                    Err(e) => {
                                        tracing::debug!(
                                            "Skipping unparseable SSE event: \
                                             {} -- data: {}",
                                            e,
                                            &data[..data.len().min(200)]
                                        );
                                        continue;
                                    }
                                }
                            }
                        } else if let Some(value) =
                            line.strip_prefix("event:")
                        {
                            event_type = value.trim().to_string();
                        } else if let Some(value) =
                            line.strip_prefix("data:")
                        {
                            if !data_buf.is_empty() {
                                data_buf.push('\n');
                            }
                            data_buf.push_str(value.trim());
                        } else if line.starts_with(':') {
                            // SSE comment -- ignore (often used as keep-alive)
                        }
                    }
                    Some(Err(e)) => {
                        return Some((
                            Err(anyhow::anyhow!("SSE byte-stream error: {}", e)),
                            (stream, event_type, data_buf),
                        ));
                    }
                    None => return None, // underlying stream closed
                }
            }
        },
    )
}

/// Parse a global-event JSON frame.
///
/// The global endpoint wraps the event:
/// ```json
/// { "directory": "/some/path", "payload": { "type": "...", ... } }
/// ```
/// We extract and return the inner `payload` as an [`Event`].
pub(crate) fn parse_global_event_data(data: &str) -> Result<Event> {
    let v: serde_json::Value =
        serde_json::from_str(data).context("invalid JSON in global SSE data")?;

    if let Some(payload) = v.get("payload") {
        let event: Event = serde_json::from_value(payload.clone())
            .context("failed to parse global event payload as Event")?;
        return Ok(event);
    }

    // Fallback: treat the entire object as an Event (defensive).
    serde_json::from_value(v)
        .context("failed to parse global event data as Event")
}

// ===========================================================================
// Byte-stream to line-stream adapter
// ===========================================================================

/// Convert a chunked byte stream into a stream of individual lines, splitting
/// on `\n` (with optional trailing `\r` stripped).
fn byte_stream_to_lines<S>(
    byte_stream: S,
) -> impl Stream<Item = Result<String, reqwest::Error>> + Send
where
    S: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send + 'static,
{
    futures::stream::unfold(
        (
            Box::pin(byte_stream)
                as Pin<
                    Box<
                        dyn Stream<
                                Item = Result<bytes::Bytes, reqwest::Error>,
                            > + Send,
                    >,
                >,
            String::new(),
        ),
        |(mut stream, mut buffer): (Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>, String)| async move {
            loop {
                // Emit a line if we already have one buffered.
                if let Some(pos) = buffer.find('\n') {
                    let line =
                        buffer[..pos].trim_end_matches('\r').to_string();
                    buffer = buffer[pos + 1..].to_string();
                    return Some((Ok(line), (stream, buffer)));
                }

                // Read the next chunk from the network.
                match stream.next().await {
                    Some(Ok(chunk)) => {
                        buffer.push_str(&String::from_utf8_lossy(&chunk));
                    }
                    Some(Err(e)) => {
                        return Some((Err(e), (stream, buffer)));
                    }
                    None => {
                        // Stream ended; flush remaining data as a final line.
                        if !buffer.is_empty() {
                            let line = std::mem::take(&mut buffer);
                            return Some((Ok(line), (stream, buffer)));
                        }
                        return None;
                    }
                }
            }
        },
    )
}
