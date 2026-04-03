use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_writer::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::fs::{File, create_dir_all};
use std::path::PathBuf;

pub struct ParquetFileHandler {
    base_path: PathBuf,
}

impl ParquetFileHandler {
    pub fn new(session_id: &str) -> Self {
        let path = PathBuf::from(format!("data/session_{}", session_id));
        create_dir_all(&path).expect("Impossible de créer le dossier de stockage");
        Self { base_path: path }
    }

    pub async fn save_batch(&self, batch: RecordBatch, part_id: usize) {
        let file_path = self.base_path.join(format!("part_{}.parquet", part_id));
        let schema = batch.schema();

        // On délègue l'écriture à un thread dédié pour ne pas bloquer l'async runtime
        tokio::task::spawn_blocking(move || {
            let file = File::create(file_path).expect("Échec création fichier");

            // Configuration de la compression (Zstd est un excellent compromis)
            let props = WriterProperties::builder()
                .set_compression(parquet::basic::Compression::ZSTD(Default::default()))
                .build();

            let mut writer = ArrowWriter::try_new(file, schema, Some(props))
                .expect("Échec création writer");

            writer.write(&batch).expect("Échec écriture du batch");
            writer.close().expect("Échec fermeture du writer");
        })
            .await
            .expect("Le thread d'écriture a paniqué");
    }
}