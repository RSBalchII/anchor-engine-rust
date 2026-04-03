//! Anchor Engine binary - HTTP server for knowledge management.
//!
//! Usage:
//! ```bash
//! anchor-engine --port 3160 --db-path ./anchor.db
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::{self, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use std::fs::File;
use std::io::{Read, Write};
use std::fs;
use tracing::error;

use anchor_engine::{Database, AnchorService, start_server, Config, WatchdogService, IngestionService, AutoSynonymGenerator};

/// Truncate log file to last N lines
fn truncate_log_file(path: &Path, max_lines: usize) -> std::io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    
    // Read all lines
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    
    let lines: Vec<&str> = content.lines().collect();
    
    // Keep only last max_lines
    if lines.len() > max_lines {
        let start = lines.len() - max_lines;
        let truncated = lines[start..].join("\n");
        
        // Write back
        let mut file = File::create(path)?;
        file.write_all(truncated.as_bytes())?;
        file.write_all(b"\n")?;
    }
    
    Ok(())
}

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

    // Ensure logs directory exists
    let logs_dir = PathBuf::from("logs");
    if let Err(e) = fs::create_dir_all(&logs_dir) {
        eprintln!("Failed to create logs directory: {}", e);
    }
    
    // Truncate existing log file to last 3000 lines
    let log_path = logs_dir.join("anchor-engine.log");
    if let Err(e) = truncate_log_file(log_path.as_path(), 3000) {
        eprintln!("Failed to truncate log file: {}", e);
    }

    // Initialize logging with rolling file appender
    let log_level = if args.verbose { "debug" } else { "info" };
    
    // Create rolling file appender
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "logs",
        "anchor-engine.log",
    );
    
    // Create a non-blocking writer
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    // Initialize both console and file logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive(log_level.parse()?)
                .add_directive("tower_http=info".parse()?)
        )
        .with_writer(non_blocking)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    tracing::info!("╔═══════════════════════════════════════════════════════════╗");
    tracing::info!("║     Anchor Engine v{} - Starting                    ║", anchor_engine::VERSION);
    tracing::info!("╚═══════════════════════════════════════════════════════════╝");
    tracing::info!("Database path: {:?}", args.db_path);
    tracing::info!("HTTP port: {}", args.port);
    tracing::info!("Logs: logs/anchor-engine.log (rolling, max 3000 lines)");

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

    // Load configuration with environment variable overrides
    tracing::info!("Loading configuration...");
    let config = Config::load_with_env_overrides().unwrap_or_default();
    
    // Validate configuration
    if let Err(e) = config.validate() {
        tracing::warn!("Configuration validation warning: {}", e);
    }
    
    tracing::info!("Inbox: {:?}", config.inbox_path());
    tracing::info!("External Inbox: {:?}", config.external_inbox_path());
    tracing::info!("Mirrored Brain: {:?}", config.mirrored_brain_path());

    // Build watch paths from config
    let mut watch_paths = vec![config.paths.notebook.clone()];
    watch_paths.extend(config.watcher.extra_paths.iter().cloned());
    tracing::info!("Watch paths: {:?}", watch_paths);

    // Create ingestion service with config
    let ingestion_service = IngestionService::new(db.clone(), config.clone());

    // Create watchdog service
    let watchdog = WatchdogService::from_config(
        &config,
        Arc::new(RwLock::new(ingestion_service)),
    );

    // Create service with pointer-only storage
    let mirror_dir = config.mirrored_brain_path();
    tracing::info!("Mirror directory: {:?}", mirror_dir);

    let service = AnchorService::new(db.clone(), mirror_dir, config.clone())
        .expect("Failed to create AnchorService with storage");
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
    for path in &watch_paths {
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
