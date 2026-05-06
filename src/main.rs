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
                eprintln!("loom init failed: {e}");
                exit(1);
            }
        }
        Some("serve") | None                       => {
            server::run_server().await;
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
    println!("loom — Knowledge Loom MCP server");
    println!();
    println!("USAGE:");
    println!("  loom               Start MCP stdio server (same as 'loom serve')");
    println!("  loom serve         Start MCP stdio server");
    println!("  loom init [dir]    Initialize loom in a directory (default: current dir)");
    println!("  loom help          Show this message");
    println!();
    println!("ENVIRONMENT:");
    println!("  KB_ROOT            Root path for knowledge base (required for serve)");
    println!("  BRAINJAR_PATH      Path to brainjar binary (optional, enables loom_search_smart)");
}
