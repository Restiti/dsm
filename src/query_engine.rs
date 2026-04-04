use datafusion::prelude::*;

pub struct QueryEngine {
    ctx: SessionContext,
}

impl QueryEngine {
    pub fn new() -> Self {
        Self {
            ctx: SessionContext::new(),
        }
    }

    pub async fn register_telemetry_data(&self, base_path: &str) -> datafusion::error::Result<()> {
        let glob_path = format!("{}/**/*.parquet", base_path.trim_end_matches('/'));

        println!("Enregistrement du chemin : {}", glob_path);

        self.ctx.register_parquet( "telemetry", &glob_path, ParquetReadOptions::default() ).await?;

        // Petite vérification pour déboguer
        let df = self.ctx.table("telemetry").await?;
        println!("Table 'telemetry' prête. Nombre de colonnes : {}", df.schema().fields().len());

        Ok(())
    }

    pub async fn query(&self, sql: &str) -> datafusion::error::Result<()> {
        let df = self.ctx.sql(sql).await?;

        // Affiche les 20 premières lignes du résultat
        df.show().await?;
        Ok(())
    }
}