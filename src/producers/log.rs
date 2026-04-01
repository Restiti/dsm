use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::time::{interval, Duration};
use crate::models::{Message, SensorData};

pub async fn run(tx: Sender<Message>) {
    // 10 Hz = 100ms
    let mut ticker = interval(Duration::from_millis(100));

    loop {
        ticker.tick().await;

        let log_payload = SensorData::Log(String::from("Système OK - Ventilation active "));

        let msg = Message::new(Arc::from("Log"), log_payload);

        if tx.send(msg).await.is_err() {
            break;
        }
    }
}