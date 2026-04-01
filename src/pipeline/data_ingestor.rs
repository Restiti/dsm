// src/pipeline/data_ingestor.rs
use tokio::sync::mpsc::Receiver;
// Importation cruciale : on a besoin des deux !
use crate::models::{Message, SensorData};

pub struct DataIngestor;

impl DataIngestor {
    // On reçoit le "Message" (l'enveloppe avec timestamp et source_id)
    pub async fn process(mut rx: Receiver<Message>) {
        tracing::info!("Démarrage de l'ingestion...");

        while let Some(msg) = rx.recv().await {
            // C'est ICI qu'on utilise SensorData défini dans models.rs
            // On fait un match sur le CHAMP 'data' du message
            match msg.data {
                SensorData::IMU { x, y, z } => {
                    tracing::info!(
                        target: "sensor::imu",
                        id = %msg.source_id, // On utilise enfin les métadonnées !
                        time = %msg.timestamp,
                        x, y, z
                    );
                }
                SensorData::GPS { lat, lon } => {
                    tracing::info!(target: "sensor::gps", id = %msg.source_id, lat, lon);
                }
                SensorData::Log(text) => {
                    tracing::info!(target: "system::log", msg = %text);
                }
            }
        }
    }
}