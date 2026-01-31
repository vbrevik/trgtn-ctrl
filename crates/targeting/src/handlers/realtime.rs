// Real-time Updates Handlers
// SSE endpoint for real-time targeting updates

use axum::{
    extract::Extension,
    response::sse::{Event, Sse},
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use crate::features::targeting::services::realtime::RealtimeService;

/// SSE endpoint for real-time targeting updates
/// GET /api/targeting/events
pub async fn events_stream_handler(
    Extension(realtime_service): Extension<Arc<RealtimeService>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = realtime_service.subscribe();

    // Convert broadcast receiver to stream
    let stream = BroadcastStream::new(rx)
        .filter_map(|result| {
            match result {
                Ok(update) => {
                    let event = Event::default()
                        .data(serde_json::to_string(&update).unwrap_or_default());
                    Some(Ok(event))
                }
                Err(_) => {
                    // Skip any errors (lagged or closed)
                    None
                }
            }
        });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
