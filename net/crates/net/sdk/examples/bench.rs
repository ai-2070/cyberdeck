//! Throughput benchmark — saturate the bus and measure events/sec.
//!
//! Run in release mode for accurate numbers:
//! ```
//! cargo run --release --example bench
//! ```

use std::time::Instant;

use net_sdk::{Backpressure, Net};

#[tokio::main(flavor = "current_thread")]
async fn main() -> net_sdk::error::Result<()> {
    let node = Net::builder()
        .shards(4)
        .backpressure(Backpressure::DropOldest)
        .memory()
        .build()
        .await?;

    let payload = r#"{"sensor":"lidar","range_m":12.5,"ts":1700000000}"#;
    let warmup = 100_000;
    let count = 2_000_000;

    // Warm up — let the allocator and branch predictor settle.
    print!("warming up ({} events)...", warmup);
    for _ in 0..warmup {
        let _ = node.emit_str(payload);
    }
    println!(" done\n");

    // --- Single-event ingestion ---
    let start = Instant::now();
    for _ in 0..count {
        let _ = node.emit_str(payload);
    }
    let elapsed = start.elapsed();

    let eps = count as f64 / elapsed.as_secs_f64();
    let ns_per = elapsed.as_nanos() as f64 / count as f64;

    println!("single-event ingestion (emit_str)");
    println!("  events:     {}", count);
    println!("  elapsed:    {:.2?}", elapsed);
    println!("  throughput: {:.2}M events/sec", eps / 1_000_000.0);
    println!("  latency:    {:.0} ns/event", ns_per);

    // --- Batch ingestion ---
    let batch_size = 1000;
    let batches = count / batch_size;
    let batch: Vec<bytes::Bytes> = (0..batch_size)
        .map(|_| bytes::Bytes::from_static(payload.as_bytes()))
        .collect();

    let start = Instant::now();
    for _ in 0..batches {
        node.emit_raw_batch(batch.clone());
    }
    let elapsed = start.elapsed();

    let total = batches * batch_size;
    let eps = total as f64 / elapsed.as_secs_f64();
    let ns_per = elapsed.as_nanos() as f64 / total as f64;

    println!("\nbatch ingestion (emit_raw_batch, batch_size={})", batch_size);
    println!("  events:     {}", total);
    println!("  elapsed:    {:.2?}", elapsed);
    println!("  throughput: {:.2}M events/sec", eps / 1_000_000.0);
    println!("  latency:    {:.0} ns/event", ns_per);

    let stats = node.stats();
    println!(
        "\ntotal ingested: {} (including {} warmup)",
        stats.events_ingested, warmup
    );

    node.shutdown().await?;
    Ok(())
}
