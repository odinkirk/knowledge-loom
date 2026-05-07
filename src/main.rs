use std::env::args;
use std::process::exit;

mod vault;
mod bm25;
mod embed;
mod index;
mod search;
mod graph;
mod edits;
mod brainjar;
mod maintenance;
mod init;
mod server;

#[tokio::main]
async fn main() {
    match args().nth(1).as_deref() {
        Some("init")                               => {
            if let Err(e) = init::run_init(args().skip(1)) {
                eprintln!("knowledge-loom init failed: {e}");
                exit(1);
            }
        }
        Some("serve") | None                       => {
            match server::run_server().await {
                Ok(_) => {}
                Err(e) => {
                    // Connection closed errors are expected in stdio mode when client disconnects
                    let error_msg = e.to_string();
                    if error_msg.contains("connection closed") || error_msg.contains("ConnectionClosed") {
                        // Exit gracefully - this is expected behavior
                        std::process::exit(0);
                    } else {
                        eprintln!("knowledge-loom serve failed: {e}");
                        exit(1);
                    }
                }
            }
        }
        Some("help") | Some("--help") | Some("-h") => {
            print_usage();
        }
        Some(other) => {
            eprintln!("unknown command: {other}");
            print_usage();
            exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("knowledge-loom — Knowledge Loom MCP server");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  loom               Start MCP stdio server (same as 'loom serve')");
    eprintln!("  loom serve         Start MCP stdio server");
    eprintln!("  loom init [dir]    Initialize knowledge-loom in a directory (default: current dir)");
    eprintln!("  loom help          Show this message");
    eprintln!();
    eprintln!("ENVIRONMENT:");
    eprintln!("  KB_ROOT            Root path for knowledge base (required for serve)");
    eprintln!("  BRAINJAR_PATH      Path to brainjar binary (optional, enables loom_search_smart)");
}
