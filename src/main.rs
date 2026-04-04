mod models;
mod producers; // Rust cherche producers/mod.rs
mod pipeline;
mod processing;
mod query_engine;

use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use crate::models::BackpressureStrategy;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let token = CancellationToken::new();
    let (tx, rx) = mpsc::channel(100);
    let metrics = Arc::new(models::Metrics::new());

    // On lance les modules
    tokio::spawn(producers::imu::run(tx.clone(), BackpressureStrategy::Drop, token.clone(), Arc::clone(&metrics)) );
    tokio::spawn(producers::gps::run(tx.clone(), BackpressureStrategy::Drop, token.clone(), Arc::clone(&metrics)));
    tokio::spawn(producers::log::run(tx.clone(), BackpressureStrategy::Block, token.clone(), Arc::clone(&metrics)));

    // Simulation d'un signal d'arrêt (ex: Ctrl+C ou après 10 secondes)
    let cl_token = token.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        println!("\n[Main] Signal d'arrêt reçu, fermeture en cours...");
        cl_token.cancel();
    });

    // TRÈS IMPORTANT : On drop le tx du main.
    // Le canal ne se fermera que lorsque TOUS les tx (ceux des spawns) seront détruits.
    drop(tx);

    // Lancement du consommateur
    let ingestor_handle = tokio::spawn(async move {
        pipeline::data_ingestor::DataIngestor::process(rx, metrics).await;
    });

    // On attend que les fichiers soient fermés
    ingestor_handle.await.unwrap();

    let engine = query_engine::QueryEngine::new();

    // On pointe vers le dossier parent qui contient toutes les sessions
    if let Err(e) = engine.register_telemetry_data("./data/").await {
        eprintln!("Erreur lors de l'enregistrement des données : {}", e);
        return;
    }

    // Exemple de requête analytique
    let sql = "
        SELECT
            source_id,
            COUNT(*) as nb_messages,
            AVG(gps_lat) as avg_lat
        FROM telemetry
        GROUP BY source_id
    ";

    if let Err(e) = engine.query(sql).await {
        eprintln!("Erreur de requête : {}", e);
    }

}