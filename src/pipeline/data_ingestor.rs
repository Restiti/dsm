use std::sync::Arc;
// src/pipeline/data_ingestor.rs
use tokio::sync::mpsc::Receiver;
// Importation cruciale : on a besoin des deux !
use crate::models::{Message, Metrics, SensorData};

pub struct DataIngestor;

impl DataIngestor {
    // On reçoit le "Message" (l'enveloppe avec timestamp et source_id)
    pub async fn process(mut rx: Receiver<Message>, metrics: Arc<Metrics>) {
        tracing::info!("Démarrage de l'ingestion...");
        let mut count = 0;
        while let Some(msg) = rx.recv().await {
            count += 1;
            // Toutes les 1000 réceptions, on fait un bilan de santé
            if count % 1000 == 0 {
                let (sent, dropped) = metrics.get_stats();
                let loss_rate = (dropped as f64 / (sent + dropped) as f64) * 100.0;

                tracing::warn!(
                    target: "system::health",
                    "Stats: Recus={}, Droppés={}, Perte={:.2}%",
                    sent, dropped, loss_rate
                );
            }

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