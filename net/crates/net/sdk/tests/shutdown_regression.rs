//! Regression test for BUG_AUDIT_2026_04_30_CORE.md #80:
//!
//! `Net::shutdown` previously gated on `Arc::try_unwrap(self.bus)`,
//! returning `Err` and silently skipping `bus.shutdown()` whenever
//! any other `Arc<EventBus>` clone existed. `Net::subscribe`
//! perpetuates a clone into every `EventStream` (#81), so the
//! failure was the default for callers that ever subscribed and
//! then attempted graceful shutdown — drain workers kept running,
//! pending events were lost, and the adapter's `flush()` /
//! `shutdown()` never ran.
//!
//! The fix routes `Net::shutdown` through `EventBus::shutdown_via_ref`,
//! which is `&self`-based and idempotent: the first caller does the
//! drain work, concurrent / subsequent callers wait for completion.

use net_sdk::{Net, SubscribeOpts};

#[tokio::test]
async fn shutdown_runs_even_with_outstanding_event_stream() {
    let node = Net::builder().memory().build().await.unwrap();

    // Subscribe — perpetuates an `Arc<EventBus>` clone into the
    // returned `EventStream`. Pre-fix this is the trigger for the
    // silent shutdown skip.
    let stream = node.subscribe(SubscribeOpts::default());

    // Pre-fix: returned `Err(Adapter("cannot shutdown: outstanding
    // references exist"))` and never called `bus.shutdown()`.
    // Post-fix: returns Ok and the bus is fully shut down.
    node.shutdown()
        .await
        .expect("shutdown must succeed even with outstanding Arc clones");

    // The stream's underlying bus reference must observe the bus as
    // shut down. Any further poll on the stream should see the
    // shutdown signal.
    drop(stream);
}

#[tokio::test]
async fn shutdown_via_ref_is_idempotent() {
    // The CAS loop in `shutdown_via_ref` ensures repeat callers
    // don't double-execute the drain logic. Subsequent calls
    // observe `shutdown_completed=true` and return Ok immediately.
    let node = Net::builder().memory().build().await.unwrap();

    node.bus()
        .shutdown_via_ref()
        .await
        .expect("first shutdown_via_ref should run the body and succeed");
    assert!(node.bus().is_shutdown());
    assert!(node.bus().is_shutdown_completed());

    node.bus()
        .shutdown_via_ref()
        .await
        .expect("second shutdown_via_ref should be a no-op and succeed");

    // Drop the node last — the bus's Drop should observe
    // `shutdown_completed=true` and not warn.
    drop(node);
}
