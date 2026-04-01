use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::time::{interval, Duration};
use crate::models::{BackpressureStrategy, Message, SensorData};

pub async fn run(tx: Sender<Message>, strategy: BackpressureStrategy) {
    // 200 Hz = 5ms
    let mut ticker = interval(Duration::from_millis(5));

    loop {
        ticker.tick().await;

        // On simule une lecture physique
        let imu_payload = SensorData::IMU { x: 0.1, y: 9.81, z: -0.2 };

        // On enveloppe dans le Message
        let msg = Message::new(Arc::from("imu_primary"), imu_payload);

        if strategy.send(&tx, msg).await.is_err() {
            break;
        }
    }
}