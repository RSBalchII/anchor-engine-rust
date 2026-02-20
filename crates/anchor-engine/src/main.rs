//! Anchor Engine binary - HTTP server for knowledge management.
//!
//! Usage:
//! ```bash
//! anchor-engine --port 3160 --db-path ./anchor.db
//! ```

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::{self, EnvFilter};
use tracing::error;

use anchor_engine::{Database, AnchorService, start_server, Config, WatchdogService, IngestionService, IngestionConfig, AutoSynonymGenerator};

/// CLI arguments.
#[derive(Debug)]
struct Args {
    port: u16,
    db_path: PathBuf,
    verbose: bool,
}

impl Args {
    fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        
        let mut port = 3160;
        let mut db_path = PathBuf::from("./anchor.db");
        let mut verbose = false;
        
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--port" | "-p" => {
                    if i + 1 < args.len() {
                        port = args[i + 1].parse().unwrap_or(3160);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--db-path" | "-d" => {
                    if i + 1 < args.len() {
                        db_path = PathBuf::from(&args[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                "--verbose" | "-v" => {
                    verbose = true;
                    i += 1;
                }
                "--help" | "-h" => {
                    println!("Anchor Engine - Knowledge Database Server");
                    println!();
                    println!("Usage: anchor-engine [OPTIONS]");
                    println!();
                    println!("Options:");
                    println!("  -p, --port <PORT>      HTTP server port (default: 3160)");
                    println!("  -d, --db-path <PATH>   Database file path (default: ./anchor.db)");
                    println!("  -v, --verbose          Enable verbose logging");
                    println!("  -h, --help             Print help");
                    std::process::exit(0);
                }
                _ => {
                    i += 1;
                }
            }
        }
        
        Self { port, db_path, verbose }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(log_level.parse()?)
                .add_directive("tower_http=info".parse()?)  // For HTTP request logging
        )
        .init();

    tracing::info!("Starting Anchor Engine v{}", anchor_engine::VERSION);
    tracing::info!("Database path: {:?}", args.db_path);
    tracing::info!("HTTP port: {}", args.port);

    // Ensure parent directory exists
    if let Some(parent) = args.db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Initialize database
    tracing::info!("Opening database...");
    let db = Database::open(&args.db_path)?;

    // Get stats
    let stats = db.get_stats().await?;
    tracing::info!("Database contains {} atoms, {} sources, {} tags",
                   stats.atom_count, stats.source_count, stats.tag_count);

    // Load configuration
    tracing::info!("Loading configuration...");
    let config = Config::load().unwrap_or_default();
    tracing::info!("Inbox: {:?}", config.settings.inbox_path());
    tracing::info!("External Inbox: {:?}", config.settings.external_inbox_path());
    tracing::info!("Mirrored Brain: {:?}", config.settings.mirrored_brain_path());
    tracing::info!("Watch paths: {:?}", config.settings.watch_paths);

    // Create ingestion service
    let ingestion_config = IngestionConfig {
        mirrored_brain_path: config.settings.mirrored_brain_path(),
        batch_size: config.settings.ingestion_batch_size,
        max_keywords: 10,
        min_keyword_score: 0.3,
        sanitize: true,
    };
    let ingestion_service = IngestionService::new(db.clone(), ingestion_config);

    // Create watchdog service
    let watchdog = WatchdogService::from_settings(
        &config.settings,
        Arc::new(RwLock::new(ingestion_service)),
    );

    // Create service
    let service = AnchorService::new(db.clone());
    let state: Arc<RwLock<AnchorService>> = Arc::new(RwLock::new(service));

    // Start HTTP server
    tracing::info!("Starting HTTP server on port {}", args.port);
    println!();
    println!("🚀 Anchor Engine v{} running", anchor_engine::VERSION);
    println!("   HTTP: http://localhost:{}", args.port);
    println!("   DB:   {:?}", args.db_path);
    println!();
    
    // Start watchdog service
    tracing::info!("Starting Watchdog service...");
    watchdog.start().await;
    println!("📥 Watchdog Service: ACTIVE");
    println!("   Watching:");
    for path in config.settings.all_watch_paths() {
        println!("     - {:?}", path);
    }
    println!();
    
    // Auto-generate synonym rings (Standard 111)
    tracing::info!("🔄 [Startup] Auto-generating synonym rings from data (Standard 111)...");
    let synonym_generator = AutoSynonymGenerator::new();
    let synonyms = synonym_generator.generate_all(&db).await;
    
    if !synonyms.is_empty() {
        let synonym_path = PathBuf::from("synonym-ring-auto.json");
        if let Err(e) = synonym_generator.save_synonym_rings(&synonyms, &synonym_path) {
            error!("Failed to save synonym rings: {}", e);
        }
        
        let summary_path = PathBuf::from("synonym-ring-auto-summary.md");
        if let Err(e) = synonym_generator.generate_summary(&synonyms, &summary_path) {
            error!("Failed to generate synonym summary: {}", e);
        }
        
        println!("🔄 [SynonymGenerator] Generated {} synonym rings", synonyms.len());
        println!("   Saved to: {:?}", synonym_path);
        println!("   Summary: {:?}", summary_path);
    } else {
        println!("🔄 [SynonymGenerator] No synonyms found (database may be empty)");
    }
    println!();
    
    println!("Endpoints:");
    println!("  GET  /health           - Health check");
    println!("  GET  /stats            - Database statistics");
    println!("  POST /v1/memory/search - Search knowledge base");
    println!("  POST /v1/memory/ingest - Ingest content");
    println!("  POST /v1/chat/completions - OpenAI-compatible chat");
    println!("  POST /v1/system/paths/add    - Add watch path");
    println!("  DELETE /v1/system/paths/remove - Remove watch path");
    println!();
    println!("Press Ctrl+C to stop");
    println!();

    // Run server with graceful shutdown
    let server_future = start_server(state, args.port);
    
    // Wait for Ctrl+C signal
    let shutdown_signal = tokio::signal::ctrl_c();
    
    tokio::select! {
        result = server_future => {
            if let Err(e) = result {
                tracing::error!("Server error: {}", e);
            }
        }
        _ = shutdown_signal => {
            tracing::info!("Received shutdown signal...");
        }
    }

    // Graceful shutdown: wipe database (disposable index pattern)
    tracing::info!("Wiping database (disposable index pattern)...");
    if let Err(e) = db.wipe_all_data().await {
        tracing::error!("Failed to wipe database: {}", e);
    } else {
        tracing::info!("Database wiped successfully. Content remains safe in mirrored_brain/");
    }

    tracing::info!("Anchor Engine shutdown complete");
    println!();
    println!("✅ Database wiped. Your content is safe in mirrored_brain/");
    println!("   The index will rebuild on next startup.");

    Ok(())
}
