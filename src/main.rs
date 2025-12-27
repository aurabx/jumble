mod config;
mod format;
mod protocol;
mod server;
mod tools;

use anyhow::{Context, Result};
use clap::Parser;
use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use server::Server;

/// An MCP server that provides queryable, on-demand project context to LLMs
#[derive(Parser, Debug)]
#[command(name = "jumble", version, about)]
struct Args {
    /// Root directory to scan for .jumble/project.toml files
    #[arg(long, env = "JUMBLE_ROOT")]
    root: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let root = args
        .root
        .or_else(|| env::var("JUMBLE_ROOT").ok().map(PathBuf::from))
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let mut server = Server::new(root)?;

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line.context("Failed to read from stdin")?;
        if line.is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let response_json = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
                continue;
            }
        };

        let response = server.handle_request(request);
        let response_json = serde_json::to_string(&response)?;
        writeln!(stdout, "{}", response_json)?;
        stdout.flush()?;
    }

    Ok(())
}
