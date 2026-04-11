//! Zentinel ZentinelSec Agent CLI
//!
//! Command-line interface for the pure Rust ModSecurity-compatible WAF agent.
//! Uses gRPC transport with Agent Protocol v2 for communication with Zentinel proxy.

use anyhow::Result;
use clap::Parser;
use tracing::info;

use zentinel_agent_protocol::v2::GrpcAgentServerV2;
use zentinel_agent_zentinelsec::{ZentinelSecAgent, ZentinelSecConfig};

/// Command line arguments
#[derive(Parser, Debug)]
#[command(name = "zentinel-zentinelsec-agent")]
#[command(
    about = "Pure Rust ModSecurity-compatible WAF agent for Zentinel - full OWASP CRS support without C dependencies"
)]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Args {
    /// gRPC server address (default: "0.0.0.0:50051")
    #[arg(long, default_value = "0.0.0.0:50051", env = "AGENT_GRPC_ADDRESS")]
    grpc_address: String,

    /// Paths to ModSecurity rule files (can be specified multiple times, supports glob patterns)
    #[arg(long = "rules", env = "ZENTINELSEC_RULES", value_delimiter = ',')]
    rules_paths: Vec<String>,

    /// Block mode (true) or detect-only mode (false)
    #[arg(long, default_value = "true", env = "ZENTINELSEC_BLOCK_MODE")]
    block_mode: bool,

    /// Paths to exclude from inspection (comma-separated)
    #[arg(long, env = "ZENTINELSEC_EXCLUDE_PATHS")]
    exclude_paths: Option<String>,

    /// Enable request body inspection
    #[arg(long, default_value = "true", env = "ZENTINELSEC_BODY_INSPECTION")]
    body_inspection: bool,

    /// Maximum body size to inspect in bytes (default 1MB)
    #[arg(long, default_value = "1048576", env = "ZENTINELSEC_MAX_BODY_SIZE")]
    max_body_size: usize,

    /// Enable response body inspection
    #[arg(long, default_value = "false", env = "ZENTINELSEC_RESPONSE_INSPECTION")]
    response_inspection: bool,

    /// Audit log path
    #[arg(long, default_value = "/var/log/zentinelsec_audit.json", env = "ZENTINELSEC_AUDIT_LOG_PATH")]
    audit_log_path: String,

    /// Enable writing logs to a local file
    #[arg(long, default_value = "true", env = "ZENTINELSEC_ENABLE_FILE_LOG")]
    enable_file_log: bool,

    /// Enable writing logs to standard output
    #[arg(long, default_value = "true", env = "ZENTINELSEC_ENABLE_CONSOLE_LOG")]
    enable_console_log: bool,

    /// Syslog URL
    #[arg(long, env = "ZENTINELSEC_SYSLOG_URL")]
    syslog_url: Option<String>,

    /// Enable verbose logging
    #[arg(short, long, env = "ZENTINELSEC_VERBOSE")]
    verbose: bool,
}

impl Args {
    fn to_config(&self) -> ZentinelSecConfig {
        let exclude_paths = self
            .exclude_paths
            .as_ref()
            .map(|p| p.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        ZentinelSecConfig {
            rules_paths: self.rules_paths.clone(),
            block_mode: self.block_mode,
            exclude_paths,
            body_inspection_enabled: self.body_inspection,
            max_body_size: self.max_body_size,
            response_inspection_enabled: self.response_inspection,
            audit_log_path: if self.enable_file_log { Some(self.audit_log_path.clone()) } else { None },
            syslog_url: self.syslog_url.clone(),
            enable_console_log: self.enable_console_log,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize tracing
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!(
            "{}={},zentinel_agent_zentinelsec={},zentinel_agent_protocol=info",
            env!("CARGO_CRATE_NAME"),
            log_level,
            log_level
        ))
        .json()
        .init();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        protocol = "v2",
        "Starting Zentinel ZentinelSec Agent (pure Rust ModSecurity)"
    );

    // Build configuration
    let config = args.to_config();

    // Create agent
    let agent = ZentinelSecAgent::new(config)?;

    info!(
        rules_paths = args.rules_paths.len(),
        block_mode = args.block_mode,
        body_inspection = args.body_inspection,
        response_inspection = args.response_inspection,
        max_body_size = args.max_body_size,
        "Configuration loaded"
    );

    // Start agent server using gRPC transport (v2 protocol)
    let addr: std::net::SocketAddr = args
        .grpc_address
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid gRPC address '{}': {}", args.grpc_address, e))?;

    info!(
        address = %addr,
        transport = "grpc",
        protocol = "v2",
        "Starting gRPC agent server"
    );

    let server = GrpcAgentServerV2::new("zentinel-zentinelsec", Box::new(agent));
    server
        .run(addr)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}
