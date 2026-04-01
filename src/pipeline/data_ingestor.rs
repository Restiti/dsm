use tokio::sync::mpsc::Receiver;
use crate::models::SensorData;

pub struct DataIngestor;

impl DataIngestor {
    pub async fn process(mut rx: Receiver<SensorData>) {
        tracing::info!("Démarrage de l'ingestion...");
        while let Some(msg) = rx.recv().await {
            match msg {
                SensorData::IMU { x, y, z } => tracing::info!(target: "sensor::imu", x, y, z),
                SensorData::GPS { lat, lon } => tracing::info!(target: "sensor::gps", lat, lon),
                SensorData::Log(text) => tracing::info!(target: "system::log", message = %text),
            }
        }
    }
}