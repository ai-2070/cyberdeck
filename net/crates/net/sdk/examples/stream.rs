//! Streaming — a producer emitting events at high throughput,
//! demonstrating how the bus distributes load across shards.
//!
//! ```
//! cargo run --example stream
//! ```

use std::collections::HashMap;

use net_sdk::{Backpressure, Net};

#[tokio::main(flavor = "current_thread")]
async fn main() -> net_sdk::error::Result<()> {
    let node = Net::builder()
        .shards(4)
        .backpressure(Backpressure::DropOldest)
        .memory()
        .build()
        .await?;

    // Simulate a sensor stream — 10,000 events across multiple sensors.
    let sensors = ["lidar", "radar", "camera", "imu", "gps", "temp"];
    let mut shard_counts: HashMap<u16, u64> = HashMap::new();

    for i in 0..10_000u64 {
        let sensor = sensors[(i as usize) % sensors.len()];
        let payload = format!(
            r#"{{"sensor":"{}","seq":{},"value":{:.2}}}"#,
            sensor,
            i,
            i as f64 * 0.1
        );
        let receipt = node.emit_str(&payload)?;
        *shard_counts.entry(receipt.shard_id).or_default() += 1;
    }

    // Show how events distributed across shards.
    let stats = node.stats();
    println!(
        "emitted {} events across {} shards\n",
        stats.events_ingested,
        node.shards()
    );

    println!("shard distribution:");
    let mut sorted: Vec<_> = shard_counts.iter().collect();
    sorted.sort_by_key(|&(id, _)| *id);
    for (shard_id, count) in &sorted {
        let pct = **count as f64 / stats.events_ingested as f64 * 100.0;
        let bar: String = std::iter::repeat('#').take((pct / 2.0) as usize).collect();
        println!(
            "  shard {}: {:>5} events ({:>5.1}%) {}",
            shard_id, count, pct, bar
        );
    }

    assert_eq!(stats.events_dropped, 0, "no events should be dropped");

    node.shutdown().await?;
    Ok(())
}
