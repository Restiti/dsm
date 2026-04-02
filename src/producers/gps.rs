use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;
use crate::models::{BackpressureStrategy, Message, SensorData};

pub async fn run(tx: Sender<Message>, strategy: BackpressureStrategy, token1: CancellationToken) {
    let mut ticker = interval(Duration::from_millis(100));
    let source_id: Arc<str> = Arc::from("gps_u_blox");

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let gps_payload = SensorData::GPS { lat: 48.8566, lon: 2.3522 };
                let msg = Message::new(Arc::clone(&source_id), gps_payload);

                if strategy.send(&tx, msg).await.is_err() {
                    break;
                }
            }
            _ = token1.cancelled() => {
                tracing::warn!(
                    target: "sensor::gps",
                    id = %source_id,
                    "Signal d'arrêt reçu, fermeture du producteur GPS"
                );
                break;
            }
        }
    }
}