//! Backpressure — demonstrate what happens when the ring buffer is full.
//!
//! The bus uses a fixed-capacity ring buffer per shard. When full, the
//! backpressure strategy determines what happens: drop the oldest event
//! (DropOldest), reject the new event (DropNewest), or fail the producer
//! (FailProducer).
//!
//! ```
//! cargo run --example backpressure
//! ```

use net_sdk::{Backpressure, Net};

#[tokio::main(flavor = "current_thread")]
async fn main() -> net_sdk::error::Result<()> {
    // --- DropOldest mode ---
    // When the ring buffer is full, the oldest event is silently evicted.
    // The producer never blocks or fails.
    println!("=== DropOldest (1 shard, capacity=1024) ===\n");

    let node = Net::builder()
        .shards(1)
        .buffer_capacity(1024)
        .backpressure(Backpressure::DropOldest)
        .memory()
        .build()
        .await?;

    // Push 2000 events into a 1024-slot buffer.
    for i in 0..2000 {
        node.emit_str(&format!(r#"{{"seq":{}}}"#, i))?;
    }

    let stats = node.stats();
    println!("emitted:   2000");
    println!("ingested:  {}", stats.events_ingested);
    println!("dropped:   {}", stats.events_dropped);
    println!("note: all 2000 accepted — old events silently evicted from the ring");

    node.shutdown().await?;

    // --- DropNewest mode ---
    // When the ring buffer is full, new events are rejected with an error.
    // The producer sees the rejection and can decide what to do.
    println!("\n=== DropNewest (1 shard, capacity=1024) ===\n");

    let node = Net::builder()
        .shards(1)
        .buffer_capacity(1024)
        .backpressure(Backpressure::DropNewest)
        .memory()
        .build()
        .await?;

    let mut accepted = 0u32;
    let mut rejected = 0u32;
    for i in 0..2000 {
        match node.emit_str(&format!(r#"{{"seq":{}}}"#, i)) {
            Ok(_) => accepted += 1,
            Err(_) => rejected += 1,
        }
    }

    println!("emitted:   2000");
    println!("accepted:  {}", accepted);
    println!("rejected:  {}", rejected);
    println!(
        "note: buffer filled at {}, remaining {} events rejected",
        accepted, rejected
    );

    node.shutdown().await?;

    // --- FailProducer mode ---
    // Same as DropNewest but semantically signals the producer should
    // back off rather than silently drop.
    println!("\n=== FailProducer (1 shard, capacity=1024) ===\n");

    let node = Net::builder()
        .shards(1)
        .buffer_capacity(1024)
        .backpressure(Backpressure::FailProducer)
        .memory()
        .build()
        .await?;

    let mut accepted = 0u32;
    let mut first_error = None;
    for i in 0..2000 {
        match node.emit_str(&format!(r#"{{"seq":{}}}"#, i)) {
            Ok(_) => accepted += 1,
            Err(e) => {
                if first_error.is_none() {
                    first_error = Some((i, e));
                }
            }
        }
    }

    println!("accepted:  {}", accepted);
    if let Some((seq, err)) = first_error {
        println!("first rejection at seq={}: {}", seq, err);
    }
    println!("the producer sees the error and can retry or back off");

    node.shutdown().await?;
    Ok(())
}
