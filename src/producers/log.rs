use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;
use crate::models::{BackpressureStrategy, Message, Metrics, SensorData};

pub async fn run(tx: Sender<Message>, strategy: BackpressureStrategy, token1: CancellationToken, metrics: Arc<Metrics>) {
    let mut ticker = interval(Duration::from_millis(1)); // Réduit à 1Hz pour l'exemple
    let source_id: Arc<str> = Arc::from("system_monitor");

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let log_payload = SensorData::Log(String::from("Système OK - Ventilation active"));
                let msg = Message::new(Arc::clone(&source_id), log_payload);

                if strategy.send(&tx, msg, &metrics).await.is_err() {
                    break;
                }
            }
            _ = token1.cancelled() => {
                tracing::warn!(
                    target: "system::log",
                    id = %source_id,
                    "Arrêt du module de logging système"
                );
                break;
            }
        }
    }
}