use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_writer::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::fs::{File, create_dir_all};
use std::path::PathBuf;
use arrow::datatypes::SchemaRef;

pub struct ParquetFileHandler {
    writer: Option<ArrowWriter<File>>,
    file_path: PathBuf,
    schema: SchemaRef,
}

impl ParquetFileHandler {
    pub fn new(session_id: &str, schema: SchemaRef) -> Self {
        let dir = format!("data/session_{}", session_id);
        create_dir_all(&dir).expect("Impossible de créer le dossier");

        let file_path = PathBuf::from(dir).join("data_stream.parquet");

        Self {
            writer: None,
            file_path,
            schema,
        }
    }

    pub fn write_batch(&mut self, batch: RecordBatch) {
        // Initialisation paresseuse du writer au premier batch
        if self.writer.is_none() {
            let file = File::create(&self.file_path).expect("Échec création fichier");
            let props = WriterProperties::builder()
                .set_compression(parquet::basic::Compression::ZSTD(Default::default()))
                .build();

            let writer = ArrowWriter::try_new(file, self.schema.clone(), Some(props))
                .expect("Échec création writer");
            self.writer = Some(writer);
        }

        if let Some(ref mut w) = self.writer {
            w.write(&batch).expect("Échec écriture du batch");
            // Optionnel : On peut flush sur le disque ici, mais c'est coûteux.
            // w.flush().unwrap();
        }
    }

    pub fn close(&mut self) {
        if let Some(w) = self.writer.take() {
            w.close().expect("Échec fermeture finale du writer");
            println!("[Writer] Fichier Parquet finalisé avec succès.");
        }
    }
}