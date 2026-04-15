//! Async stream-based event consumption.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::Stream;
use futures::StreamExt;

use net::consumer::Ordering;
use net::{ConsumeRequest, EventBus, Filter, StoredEvent};

use crate::error::{Result, SdkError};

/// Options for subscribing to events.
#[derive(Clone, Debug)]
pub struct SubscribeOpts {
    pub(crate) limit: usize,
    pub(crate) filter: Option<Filter>,
    pub(crate) ordering: Ordering,
    pub(crate) poll_interval: Duration,
    pub(crate) max_backoff: Duration,
}

impl Default for SubscribeOpts {
    fn default() -> Self {
        Self {
            limit: 100,
            filter: None,
            ordering: Ordering::None,
            poll_interval: Duration::from_millis(1),
            max_backoff: Duration::from_millis(100),
        }
    }
}

impl SubscribeOpts {
    /// Set the maximum number of events per poll.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set an event filter.
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Set the event ordering.
    pub fn ordering(mut self, ordering: Ordering) -> Self {
        self.ordering = ordering;
        self
    }

    /// Set the base poll interval.
    pub fn poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Set the maximum backoff interval.
    pub fn max_backoff(mut self, max: Duration) -> Self {
        self.max_backoff = max;
        self
    }
}

type PollFuture = Pin<
    Box<
        dyn Future<Output = std::result::Result<net::ConsumeResponse, net::error::ConsumerError>>
            + Send,
    >,
>;

/// An async stream of events from the event bus.
///
/// Internally polls the bus with adaptive backoff — polls tightly when
/// events are flowing, backs off when idle.
pub struct EventStream {
    bus: Arc<EventBus>,
    opts: SubscribeOpts,
    cursor: Option<String>,
    buffer: Vec<StoredEvent>,
    buffer_idx: usize,
    current_interval: Duration,
    sleep: Option<Pin<Box<tokio::time::Sleep>>>,
    inflight: Option<PollFuture>,
}

impl EventStream {
    pub(crate) fn new(bus: Arc<EventBus>, opts: SubscribeOpts) -> Self {
        let interval = opts.poll_interval;
        Self {
            bus,
            opts,
            cursor: None,
            buffer: Vec::new(),
            buffer_idx: 0,
            current_interval: interval,
            sleep: None,
            inflight: None,
        }
    }
}

impl Stream for EventStream {
    type Item = Result<StoredEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // Return buffered events first.
        if this.buffer_idx < this.buffer.len() {
            let event = this.buffer[this.buffer_idx].clone();
            this.buffer_idx += 1;
            return Poll::Ready(Some(Ok(event)));
        }

        // If we have a sleep pending, wait for it.
        if let Some(sleep) = &mut this.sleep {
            match Pin::new(sleep).poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(()) => {
                    this.sleep = None;
                }
            }
        }

        // If we have an in-flight poll, resume it.
        if this.inflight.is_none() {
            let mut request = ConsumeRequest::new(this.opts.limit);
            if let Some(cursor) = &this.cursor {
                request = request.from(cursor);
            }
            if let Some(filter) = &this.opts.filter {
                request = request.filter(filter.clone());
            }
            request = request.ordering(this.opts.ordering);

            let bus = this.bus.clone();
            this.inflight = Some(Box::pin(async move { bus.poll(request).await }));
        }

        let fut = this.inflight.as_mut().unwrap();
        match fut.as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => {
                this.inflight = None;
                Poll::Ready(Some(Err(SdkError::from(e))))
            }
            Poll::Ready(Ok(response)) => {
                this.inflight = None;
                if response.events.is_empty() {
                    // Backoff: double the interval, up to max.
                    this.current_interval = (this.current_interval * 2).min(this.opts.max_backoff);
                    this.sleep = Some(Box::pin(tokio::time::sleep(this.current_interval)));
                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {
                    // Reset backoff on activity.
                    this.current_interval = this.opts.poll_interval;
                    this.cursor = response.next_id;
                    this.buffer = response.events;
                    this.buffer_idx = 1;
                    Poll::Ready(Some(Ok(this.buffer[0].clone())))
                }
            }
        }
    }
}

/// A typed async stream that deserializes events into `T`.
pub struct TypedEventStream<T> {
    inner: EventStream,
    _marker: std::marker::PhantomData<T>,
}

impl<T: serde::de::DeserializeOwned> TypedEventStream<T> {
    pub(crate) fn new(bus: Arc<EventBus>, opts: SubscribeOpts) -> Self {
        Self {
            inner: EventStream::new(bus, opts),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: serde::de::DeserializeOwned + Unpin> Stream for TypedEventStream<T> {
    type Item = Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.poll_next_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(Some(Ok(event))) => {
                let parsed =
                    serde_json::from_slice(event.raw.as_ref()).map_err(SdkError::Serialization);
                Poll::Ready(Some(parsed))
            }
        }
    }
}
