use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

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