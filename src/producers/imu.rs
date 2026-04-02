use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;
use crate::models::{BackpressureStrategy, Message, SensorData};

pub async fn run(tx: Sender<Message>, strategy: BackpressureStrategy,  token1: CancellationToken) {
    // 200 Hz = 5ms
    let mut ticker = interval(Duration::from_millis(5));
    let source_id: Arc<str> = Arc::from("imu_primary");

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                // On simule une lecture physique
                let imu_payload = SensorData::IMU { x: 0.1, y: 9.81, z: -0.2 };

                // On enveloppe dans le Message
                let msg = Message::new(Arc::clone(&source_id), imu_payload);

                if strategy.send(&tx, msg).await.is_err() {
                    break;
                }
            }
            _ = token1.cancelled() => {
                    tracing::warn!(
                            target: "sensor::imu",
                            id = %source_id, // Utilise l'Arc<str> défini avant la boucle
                            "Signal d'arrêt reçu, fermeture du producteur"
                        );
                break;
            }
        }

    }
}