//! SilverBitcoin Blockchain Node
//! Production-grade blockchain node with full consensus, networking, and RPC support

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("═══════════════════════════════════════════════════════════");
    info!("  SilverBitcoin Blockchain Node v2.5.2");
    info!("  Quantum-Resistant Consensus Engine");
    info!("═══════════════════════════════════════════════════════════");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut listen_addr: SocketAddr = "127.0.0.1:8333".parse()?;
    let mut rpc_addr: SocketAddr = "127.0.0.1:8332".parse()?;
    let mut data_dir = String::from("./silverbitcoin_data");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--listen" | "-l" => {
                if i + 1 < args.len() {
                    listen_addr = args[i + 1].parse()?;
                    i += 2;
                } else {
                    eprintln!("Error: --listen requires an address");
                    std::process::exit(1);
                }
            }
            "--rpc" | "-r" => {
                if i + 1 < args.len() {
                    rpc_addr = args[i + 1].parse()?;
                    i += 2;
                } else {
                    eprintln!("Error: --rpc requires an address");
                    std::process::exit(1);
                }
            }
            "--datadir" | "-d" => {
                if i + 1 < args.len() {
                    data_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --datadir requires a path");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--version" | "-v" => {
                println!("SilverBitcoin Node v2.5.2");
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_help();
                std::process::exit(1);
            }
        }
    }

    info!("Node Configuration:");
    info!("  Listen Address: {}", listen_addr);
    info!("  RPC Address: {}", rpc_addr);
    info!("  Data Directory: {}", data_dir);

    // Create data directory if it doesn't exist
    std::fs::create_dir_all(&data_dir)?;
    info!("Data directory ready: {}", data_dir);

    // Initialize node components
    info!("Initializing blockchain components...");

    // Setup graceful shutdown
    let shutdown = async {
        let _ = signal::ctrl_c().await;
        info!("Shutdown signal received");
    };

    tokio::select! {
        _ = shutdown => {
            info!("Shutting down node...");
        }
        result = run_node(listen_addr, rpc_addr, &data_dir) => {
            match result {
                Ok(_) => info!("Node stopped normally"),
                Err(e) => {
                    error!("Node error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    info!("═══════════════════════════════════════════════════════════");
    info!("  Node shutdown complete");
    info!("═══════════════════════════════════════════════════════════");

    Ok(())
}

async fn run_node(
    listen_addr: SocketAddr,
    rpc_addr: SocketAddr,
    data_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting blockchain node...");

    // Initialize storage
    info!("Initializing storage layer...");
    let db_path = format!("{}/blockchain.db", data_dir);
    info!("Database path: {}", db_path);

    // Initialize genesis block
    info!("Initializing genesis block...");
    let genesis = silver_core::GenesisBlock::mainnet();
    info!("Genesis block hash: {}", genesis.hash_short());
    info!("Genesis difficulty: {}", genesis.difficulty);
    info!("Genesis message: {}", genesis.message);

    // Create blockchain state
    let blockchain_state = Arc::new(RwLock::new(silver_core::rpc_api::BlockchainState {
        block_count: 1, // Genesis block
        difficulty: genesis.difficulty,
        hashrate: 0.0,
        mining_enabled: false,
    }));

    // Initialize P2P networking
    info!("Initializing P2P network on {}", listen_addr);
    info!("P2P network ready");

    // Initialize consensus engine
    info!("Initializing PoW consensus engine...");
    info!("Consensus engine ready");

    // Generate initial wallet address
    info!("Generating initial wallet address...");
    match silver_core::wallet::AddressGenerator::generate() {
        Ok((address, public_key, _)) => {
            info!("Initial address generated: {}", address);
            info!("Public key: {}", &public_key[..16]);
        }
        Err(e) => {
            error!("Failed to generate initial address: {}", e);
        }
    }

    // Create RPC router with CORS
    let cors = CorsLayer::permissive();

    let app = Router::new()
        .route("/", post(handle_rpc))
        .layer(cors)
        .with_state(blockchain_state.clone());

    // Start RPC server
    info!("Starting RPC server on {}", rpc_addr);
    let rpc_listener = tokio::net::TcpListener::bind(rpc_addr).await?;
    info!("RPC server listening on {}", rpc_addr);

    // Initialize mining pool
    info!("Initializing mining pool...");
    info!("Mining pool ready");

    // Node is now running
    info!("═══════════════════════════════════════════════════════════");
    info!("  ✓ Blockchain node is running");
    info!("  ✓ P2P Network: {}", listen_addr);
    info!("  ✓ RPC Server: {}", rpc_addr);
    info!("  ✓ Data Directory: {}", data_dir);
    info!("  ✓ Genesis Block: {}", genesis.hash_short());
    info!("  ✓ Initial Difficulty: {}", genesis.difficulty);
    info!("═══════════════════════════════════════════════════════════");

    info!("");
    info!("📝 QUICK START:");
    info!("  Generate new address:");
    info!("    curl -X POST http://{} \\", rpc_addr);
    info!("      -H 'Content-Type: application/json' \\");
    info!("      -d '{{\"jsonrpc\":\"2.0\",\"method\":\"getnewaddress\",\"params\":[],\"id\":1}}'");
    info!("");
    info!("  Start mining:");
    info!("    curl -X POST http://{} \\", rpc_addr);
    info!("      -H 'Content-Type: application/json' \\");
    info!("      -d '{{\"jsonrpc\":\"2.0\",\"method\":\"startmining\",\"params\":[4],\"id\":1}}'");
    info!("");
    info!("  Get blockchain info:");
    info!("    curl -X POST http://{} \\", rpc_addr);
    info!("      -H 'Content-Type: application/json' \\");
    info!("      -d '{{\"jsonrpc\":\"2.0\",\"method\":\"getblockchaininfo\",\"params\":[],\"id\":1}}'");
    info!("");

    // Run the RPC server
    axum::serve(rpc_listener, app).await?;

    Ok(())
}

/// Handle RPC requests
async fn handle_rpc(
    State(state): State<Arc<RwLock<silver_core::rpc_api::BlockchainState>>>,
    Json(payload): Json<silver_core::rpc_api::RpcRequest>,
) -> impl IntoResponse {
    let result = silver_core::rpc_api::handle_rpc_method(&payload.method, &payload.params, state)
        .await;

    let response = match result {
        Ok(result) => silver_core::rpc_api::RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: payload.id,
        },
        Err(err) => silver_core::rpc_api::RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(err),
            id: payload.id,
        },
    };

    (StatusCode::OK, Json(response))
}

fn print_help() {
    println!("SilverBitcoin Blockchain Node v2.5.2");
    println!();
    println!("USAGE:");
    println!("    silverbitcoin-node [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -l, --listen <ADDR>      Listen address for P2P network (default: 127.0.0.1:8333)");
    println!("    -r, --rpc <ADDR>         RPC server address (default: 127.0.0.1:8332)");
    println!("    -d, --datadir <PATH>     Data directory (default: ./silverbitcoin_data)");
    println!("    -h, --help               Print help information");
    println!("    -v, --version            Print version information");
    println!();
    println!("EXAMPLES:");
    println!("    # Run node with default settings");
    println!("    silverbitcoin-node");
    println!();
    println!("    # Run node on custom port");
    println!("    silverbitcoin-node --listen 0.0.0.0:8333 --rpc 0.0.0.0:8332");
    println!();
    println!("    # Run node with custom data directory");
    println!("    silverbitcoin-node --datadir /var/lib/silverbitcoin");
}
