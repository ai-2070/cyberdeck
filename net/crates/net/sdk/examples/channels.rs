//! Typed channels — publish structured data and verify round-trip
//! serialization through the bus.
//!
//! ```
//! cargo run --example channels
//! ```

use net_sdk::{Backpressure, Net};
use serde::{Deserialize, Serialize};

/// A temperature reading from a sensor.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TemperatureReading {
    sensor_id: String,
    celsius: f64,
    timestamp_ms: u64,
}

/// An alert generated when a threshold is exceeded.
#[derive(Debug, Serialize, Deserialize)]
struct Alert {
    source: String,
    message: String,
    severity: u8,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> net_sdk::error::Result<()> {
    let node = Net::builder()
        .shards(4)
        .backpressure(Backpressure::DropOldest)
        .memory()
        .build()
        .await?;

    // Publish temperature readings as typed structs.
    let readings = vec![
        TemperatureReading {
            sensor_id: "A1".into(),
            celsius: 22.5,
            timestamp_ms: 1000,
        },
        TemperatureReading {
            sensor_id: "A2".into(),
            celsius: 85.3,
            timestamp_ms: 1001,
        },
        TemperatureReading {
            sensor_id: "B1".into(),
            celsius: 19.8,
            timestamp_ms: 1002,
        },
        TemperatureReading {
            sensor_id: "A1".into(),
            celsius: 91.7,
            timestamp_ms: 1003,
        },
    ];

    println!("publishing {} temperature readings:", readings.len());
    for reading in &readings {
        let r = node.emit(reading)?;
        let status = if reading.celsius > 80.0 { " [HOT]" } else { "" };
        println!(
            "  {} — {:.1} C{} -> shard {}",
            reading.sensor_id, reading.celsius, status, r.shard_id
        );
    }

    // Publish alerts as a different type through the same bus.
    let alerts = vec![
        Alert {
            source: "A2".into(),
            message: "temperature above threshold".into(),
            severity: 2,
        },
        Alert {
            source: "A1".into(),
            message: "temperature critical".into(),
            severity: 1,
        },
    ];

    println!("\npublishing {} alerts:", alerts.len());
    for alert in &alerts {
        let r = node.emit(alert)?;
        println!(
            "  [sev={}] {} — {} -> shard {}",
            alert.severity, alert.source, alert.message, r.shard_id
        );
    }

    // Demonstrate that batch emit works with typed data.
    let batch_readings: Vec<TemperatureReading> = (0..100)
        .map(|i| TemperatureReading {
            sensor_id: format!("S{}", i % 10),
            celsius: 20.0 + (i as f64) * 0.1,
            timestamp_ms: 2000 + i,
        })
        .collect();

    let batch_count = node.emit_batch(&batch_readings)?;
    println!("\nbatch emitted {} readings", batch_count);

    let stats = node.stats();
    println!(
        "\ntotal: {} events ingested, {} dropped",
        stats.events_ingested, stats.events_dropped
    );
    assert_eq!(stats.events_ingested, 106); // 4 + 2 + 100

    node.shutdown().await?;
    Ok(())
}
