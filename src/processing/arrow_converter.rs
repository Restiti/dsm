use arrow::array::*;
use arrow::datatypes::*;
use arrow::record_batch::RecordBatch;
use std::sync::Arc;
use crate::models::{Message, SensorData};

pub struct ArrowConverter {
    schema: SchemaRef,
    // Builders
    ts_builder: TimestampMillisecondBuilder,
    id_builder: StringBuilder,
    // Le builder IMU est un FixedSizeList qui contient un Float32Builder
    imu_builder: FixedSizeListBuilder<Float32Builder>,
    lat_builder: Float64Builder,
    lon_builder: Float64Builder,
    log_builder: StringBuilder,

    current_rows: usize,
}

impl ArrowConverter {
    pub fn new(capacity: usize) -> Self {
        let schema = Arc::new(Schema::new(vec![
            Field::new("timestamp", DataType::Timestamp(TimeUnit::Millisecond, None), false),
            Field::new("source_id", DataType::Utf8, false),
            Field::new("imu", DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                3
            ), true),
            Field::new("gps_lat", DataType::Float64, true),
            Field::new("gps_lon", DataType::Float64, true),
            Field::new("log_msg", DataType::Utf8, true),
        ]));

        // Initialisation des builders avec une capacité pré-allouée
        let ts_builder = TimestampMillisecondBuilder::with_capacity(capacity);
        let id_builder = StringBuilder::with_capacity(capacity, capacity * 15);

        // Configuration du builder IMU (Vecteur 3D)
        let values_builder = Float32Builder::with_capacity(capacity * 3);
        let imu_builder = FixedSizeListBuilder::new(values_builder, 3);

        let lat_builder = Float64Builder::with_capacity(capacity);
        let lon_builder = Float64Builder::with_capacity(capacity);
        let log_builder = StringBuilder::with_capacity(capacity, capacity * 50);

        Self {
            schema,
            ts_builder,
            id_builder,
            imu_builder,
            lat_builder,
            lon_builder,
            log_builder,
            current_rows: 0,
        }
    }

    pub fn add_message(&mut self, msg: Message) {
        // Colonnes communes
        self.ts_builder.append_value(msg.timestamp as i64);
        self.id_builder.append_value(&msg.source_id);

        // Colonnes spécifiques (Gestion des Nulls)
        match msg.data {
            SensorData::IMU { x, y, z } => {
                // On ajoute les 3 valeurs au builder interne
                let values = self.imu_builder.values();
                values.append_value(x);
                values.append_value(y);
                values.append_value(z);
                self.imu_builder.append(true); // On valide l'entrée IMU

                self.lat_builder.append_null();
                self.lon_builder.append_null();
                self.log_builder.append_null();
            }
            SensorData::GPS { lat, lon } => {
                let values = self.imu_builder.values();
                values.append_slice(&[0.0, 0.0, 0.0]);
                self.imu_builder.append(false); // Null pour IMU
                self.lat_builder.append_value(lat);
                self.lon_builder.append_value(lon);
                self.log_builder.append_null();
            }
            SensorData::Log(text) => {
                let values = self.imu_builder.values();
                values.append_slice(&[0.0, 0.0, 0.0]);
                self.imu_builder.append(false);
                self.lat_builder.append_null();
                self.lon_builder.append_null();
                self.log_builder.append_value(text);
            }
        }
        self.current_rows += 1;
    }

    /// Transforme les builders accumulés en un RecordBatch et réinitialise les builders
    pub fn finish(&mut self) -> RecordBatch {
        let columns: Vec<ArrayRef> = vec![
            Arc::new(self.ts_builder.finish()),
            Arc::new(self.id_builder.finish()),
            Arc::new(self.imu_builder.finish()),
            Arc::new(self.lat_builder.finish()),
            Arc::new(self.lon_builder.finish()),
            Arc::new(self.log_builder.finish()),
        ];

        self.current_rows = 0;
        RecordBatch::try_new(Arc::clone(&self.schema), columns)
            .expect("Échec de la création du RecordBatch")
    }

    pub fn rows_count(&self) -> usize {
        self.current_rows
    }
}