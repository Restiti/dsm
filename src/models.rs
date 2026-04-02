use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub enum SensorData {
    IMU { x: f32, y: f32, z: f32 },
    GPS { lat: f64, lon: f64 },
    Log(String),
}

pub struct Message {
    pub source_id: Arc<str>,       // "imu_01", "gps_primary"
    pub timestamp: u64,          // Timestamp Unix en ms (très important pour synchroniser les flux)
    pub data: SensorData,        // L'énumération contenant la donnée brute
}

impl Message {
    // Petit helper pour créer un message avec le temps actuel
    pub fn new(source_id: Arc<str>, data: SensorData) -> Self {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
        Self {
            source_id: source_id,
            timestamp: since_the_epoch.as_millis() as u64,
            data,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BackpressureStrategy {
    Block, // Utilise .send().await (ralentit le producteur)
    Drop,  // Utilise .try_send() (jette la donnée si plein)
}
impl BackpressureStrategy {
    /// Gère l'envoi d'un message selon la stratégie choisie
    pub async fn send(&self, tx: &Sender<Message>, msg: Message, metrics: &Metrics) -> Result<(), ()> {
        match self {
            BackpressureStrategy::Block => {
                if tx.send(msg).await.is_err() { return Err(()); }
                metrics.total_sent.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            BackpressureStrategy::Drop => {
                match tx.try_send(msg) {
                    Ok(_) => {
                        metrics.total_sent.fetch_add(1, Ordering::Relaxed);
                        Ok(())
                    }
                    Err(TrySendError::Full(_)) => {
                        metrics.total_dropped.fetch_add(1, Ordering::Relaxed);
                        Ok(())
                    }
                    Err(TrySendError::Closed(_)) => Err(()),
                }
            }
        }
    }
}

pub struct Metrics {
    pub total_sent: AtomicU64,
    pub total_dropped: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            total_sent: AtomicU64::new(0),
            total_dropped: AtomicU64::new(0),
        }
    }

    // Un petit helper pour lire les valeurs
    pub fn get_stats(&self) -> (u64, u64) {
        (
            self.total_sent.load(Ordering::Relaxed),
            self.total_dropped.load(Ordering::Relaxed),
        )
    }
}
