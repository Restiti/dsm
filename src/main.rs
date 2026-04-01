mod models;
mod producers; // Rust cherche producers/mod.rs
mod pipeline;

use tokio::sync::mpsc;
use crate::models::BackpressureStrategy;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let (tx, rx) = mpsc::channel(100);

    // On lance les modules
    tokio::spawn(producers::imu::run(tx.clone(), BackpressureStrategy::Drop));
    tokio::spawn(producers::gps::run(tx.clone(), BackpressureStrategy::Drop));
    tokio::spawn(producers::log::run(tx.clone(), BackpressureStrategy::Block));

    // Lancement du consommateur
    pipeline::data_ingestor::DataIngestor::process(rx).await;
}