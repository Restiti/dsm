use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tokio::time::{Instant, Duration};
use crate::models::{Message, Metrics};
use crate::pipeline::parquet_writer::ParquetFileHandler;
use crate::processing::arrow_converter::ArrowConverter;

pub struct DataIngestor;

impl DataIngestor {
    pub async fn process(mut rx: Receiver<Message>, metrics: Arc<Metrics>) {
        tracing::info!("Démarrage de l'ingestion...");
        let batch_size = 1000;
        let mut converter = ArrowConverter::new(batch_size);
        let mut last_flush = Instant::now();
        let mut total_processed = 0;
        let session_id = chrono::Utc::now().timestamp().to_string();


        let schema = converter.get_schema(); // Ajoute un getter dans ArrowConverter
        let mut writer_handler = ParquetFileHandler::new(&session_id, schema);

        while let Some(msg) = rx.recv().await {

            converter.add_message(msg);
            total_processed += 1;
            // Toutes les 1000 réceptions, on fait un bilan de santé
            if total_processed % 1000 == 0 {
                let (sent, dropped) = metrics.get_stats();
                let loss_rate = (dropped as f64 / (sent + dropped) as f64) * 100.0;
                tracing::warn!(
                    target: "system::health",
                    "Stats: Recus={}, Droppés={}, Perte={:.2}%",
                    sent, dropped, loss_rate
                );
            }

            if converter.rows_count() >= batch_size || last_flush.elapsed() >= Duration::from_secs(1) {
                if converter.rows_count() > 0 {
                    let batch = converter.finish();
                    tracing::info!(
                        target: "pipeline::arrow",
                        "RecordBatch créé : {} lignes, {} colonnes",
                        batch.num_rows(), batch.num_columns()
                    );

                    writer_handler.write_batch(batch);

                    last_flush = Instant::now();
                }
            }
        }
        writer_handler.close();
    }
}