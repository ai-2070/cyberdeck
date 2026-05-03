//! Regression coverage for `net_poll`'s buffer-size handling.
//!
//! Pre-fix `net_poll` always invoked `bus.poll(request)` first and
//! only checked the buffer size after the response was already
//! serialized; if the buffer was too small, the function returned
//! `BufferTooSmall` and dropped the response. A caller that
//! trusted the returned `next_id` from a previous call could
//! advance their cursor past unread events.
//!
//! Post-fix:
//! - Buffers smaller than `MIN_RESPONSE_BUFFER` (256 bytes) are
//!   rejected up front, before any adapter work happens.
//! - When a polled response is too large for the caller's buffer,
//!   a minimal fallback JSON is written that echoes the original
//!   cursor as `next_id` (so the caller's retry re-polls the same
//!   range against an idempotent adapter).
//!
//! The default FFI handle uses the noop adapter, which never
//! returns any events from `poll_shard`. That makes the post-poll
//! overflow path unreachable without a real adapter, so this test
//! pins the pre-poll minimum-buffer check (which is what catches
//! the degenerate "tiny buffer" misuse) and the empty-response
//! happy path.

use std::os::raw::c_char;
use std::ptr;

use net::ffi::{net_init, net_poll, net_shutdown};

const NET_ERR_BUFFER_TOO_SMALL: i32 = -7;

#[test]
fn net_poll_rejects_buffers_below_minimum_without_polling() {
    let handle = net_init(ptr::null());
    assert!(!handle.is_null(), "net_init failed");

    // Buffer of 100 bytes is below the 256-byte minimum and is
    // pre-emptively rejected. A pre-fix run polled the bus first
    // and dropped the response on this path; post-fix the rejection
    // happens before any cursor work.
    let mut buf = vec![0u8; 100];
    let code = net_poll(
        handle,
        ptr::null::<c_char>(),
        buf.as_mut_ptr() as *mut c_char,
        buf.len(),
    );
    assert_eq!(
        code, NET_ERR_BUFFER_TOO_SMALL,
        "100-byte buffer must be rejected with BufferTooSmall, got {}",
        code,
    );

    // Even tinier buffer — same rejection.
    let mut tiny = vec![0u8; 10];
    let code = net_poll(
        handle,
        ptr::null::<c_char>(),
        tiny.as_mut_ptr() as *mut c_char,
        tiny.len(),
    );
    assert_eq!(
        code, NET_ERR_BUFFER_TOO_SMALL,
        "10-byte buffer must be rejected with BufferTooSmall, got {}",
        code,
    );

    let _ = net_shutdown(handle);
}

#[test]
fn net_poll_accepts_buffers_at_or_above_minimum() {
    let handle = net_init(ptr::null());
    assert!(!handle.is_null(), "net_init failed");

    // 4 KB comfortably exceeds the minimum, and the noop adapter
    // returns an empty event list so the response easily fits.
    let mut buf = vec![0u8; 4096];
    let code = net_poll(
        handle,
        b"{\"limit\": 10}\0".as_ptr() as *const c_char,
        buf.as_mut_ptr() as *mut c_char,
        buf.len(),
    );
    assert!(
        code >= 0,
        "expected positive byte count from successful empty poll, got {}",
        code,
    );

    // The first `code` bytes of the buffer must be a valid JSON
    // response containing at least an `events` array.
    let written = &buf[..code as usize];
    let s = std::str::from_utf8(written).expect("response is not UTF-8");
    assert!(
        s.contains("\"events\""),
        "response missing events field: {}",
        s,
    );

    let _ = net_shutdown(handle);
}
