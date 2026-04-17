//! Hello World — emit events into the mesh and verify ingestion.
//!
//! The in-memory adapter is a speed buffer with no persistence backend,
//! so this example demonstrates the ingestion path: emit events, check
//! stats, and confirm everything was accepted.
//!
//! ```
//! cargo run --example hello
//! ```

use net_sdk::{Backpressure, Net};

#[tokio::main(flavor = "current_thread")]
async fn main() -> net_sdk::error::Result<()> {
    let node = Net::builder()
        .shards(4)
        .backpressure(Backpressure::DropOldest)
        .memory()
        .build()
        .await?;

    // Emit structured events (serialized via serde).
    let r = node.emit(&serde_json::json!({"sensor": "lidar", "range_m": 12.5}))?;
    println!("lidar  -> shard {} at ts {}", r.shard_id, r.timestamp);

    let r = node.emit(&serde_json::json!({"sensor": "radar", "range_m": 45.0}))?;
    println!("radar  -> shard {} at ts {}", r.shard_id, r.timestamp);

    let r = node.emit(&serde_json::json!({"sensor": "camera", "objects": 3}))?;
    println!("camera -> shard {} at ts {}", r.shard_id, r.timestamp);

    // Raw string path (fastest — no serialization overhead).
    let r = node.emit_str(r#"{"sensor": "imu", "accel_g": 1.02}"#)?;
    println!("imu    -> shard {} at ts {}", r.shard_id, r.timestamp);

    // Raw bytes path.
    let r = node.emit_raw(b"{\"sensor\": \"gps\", \"lat\": 35.6762}" as &[u8])?;
    println!("gps    -> shard {} at ts {}", r.shard_id, r.timestamp);

    // Check stats.
    let stats = node.stats();
    println!(
        "\n{} events ingested, {} dropped",
        stats.events_ingested, stats.events_dropped
    );
    assert_eq!(stats.events_ingested, 5);
    assert_eq!(stats.events_dropped, 0);
    println!("all events accepted across {} shards", node.shards());

    node.shutdown().await?;
    Ok(())
}
