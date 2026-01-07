mod config;
mod format;
mod memory;
mod protocol;
mod server;
mod setup;
mod tools;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::env;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use server::Server;

/// An MCP server that provides queryable, on-demand project context to LLMs
#[derive(Parser, Debug)]
#[command(name = "jumble", version, about)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Root directory to scan for .jumble/project.toml files (server mode only)
    #[arg(long, env = "JUMBLE_ROOT", global = true)]
    root: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the MCP server (default if no subcommand specified)
    Server,
    
    /// Setup AI agent integrations
    Setup {
        #[command(subcommand)]
        agent: SetupCommands,
    },
}

#[derive(Subcommand, Debug)]
enum SetupCommands {
    /// Setup Warp integration by creating/updating WARP.md
    Warp {
        /// Force update even if jumble section already exists
        #[arg(short, long)]
        force: bool,
    },
    
    /// Setup Claude Desktop integration
    Claude {
        /// Use global config (~/.claude) instead of project .claude directory
        #[arg(short, long)]
        global: bool,
    },
    
    /// Setup Cursor integration
    Cursor {
        /// Use global config (~/.cursor) instead of project .cursor directory
        #[arg(short, long)]
        global: bool,
    },
    
    /// Setup Windsurf integration
    Windsurf {
        /// Use global config (~/.codeium/windsurf) instead of project .windsurf directory
        #[arg(short, long)]
        global: bool,
    },
    
    /// Setup Codex integration
    Codex {
        /// Use global config (~/.codex) instead of project .codex directory
        #[arg(short, long)]
        global: bool,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let root = args
        .root
        .or_else(|| env::var("JUMBLE_ROOT").ok().map(PathBuf::from))
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    
    match args.command {
        Some(Commands::Server) | None => {
            // Run MCP server (default mode)
            run_server(root)
        }
        Some(Commands::Setup { agent }) => {
            match agent {
                SetupCommands::Warp { force } => {
                    setup::setup_warp(&root, force)
                }
                SetupCommands::Claude { global } => {
                    setup::setup_claude(&root, global)
                }
                SetupCommands::Cursor { global } => {
                    setup::setup_cursor(&root, global)
                }
                SetupCommands::Windsurf { global } => {
                    setup::setup_windsurf(&root, global)
                }
                SetupCommands::Codex { global } => {
                    setup::setup_codex(&root, global)
                }
            }
        }
    }
}

fn run_server(root: PathBuf) -> Result<()> {
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
