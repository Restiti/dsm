use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::time::{interval, Duration};
use crate::models::{Message, SensorData};

pub async fn run(tx: Sender<Message>) {
    // 10 Hz = 100ms
    let mut ticker = interval(Duration::from_millis(100));

    loop {
        ticker.tick().await;

        let gps_payload = SensorData::GPS { lat: 48.8566, lon: 2.3522 };
        let msg = Message::new(Arc::from("gps_u_blox"), gps_payload);

        if let Err(tokio::sync::mpsc::error::TrySendError::Full(_)) = tx.try_send(msg) {
            tracing::warn!("IMU dropping frame");
        }
    }
}