use std::env::args;
use std::process::exit;

mod bm25;
mod chunks;
mod daemon;
mod download;
mod edits;
mod embed;
mod graph;
mod index;
mod init;
mod install;
mod maintenance;
mod model;
mod platforms;
mod search;
mod server;
mod shell;
mod vault;
mod web;

#[tokio::main]
async fn main() {
    match args().nth(1).as_deref() {
        Some("init") => {
            if let Err(e) = init::run_init_async(args().skip(1).collect()).await {
                eprintln!("knowledge-loom init failed: {e}");
                exit(1);
            }
        }
        Some("install") => {
            let force = knowledge_loom::cli::args::parse_flag("force", Some("f"));
            let kb_root = std::env::var("KB_ROOT")
                .unwrap_or_else(|_| ".".to_string())
                .into();
            match install::run_install(kb_root, force).await {
                Ok(summary) => {
                    println!(
                        "Installed {} ({:.1} MB) to {}",
                        summary.model_version,
                        summary.size_bytes as f64 / 1_000_000.0,
                        summary.target_location
                    );
                    println!("Checksum: {}", summary.checksum);
                }
                Err(install::InstallError::AlreadyInstalled) => {
                    println!(
                        "fastembed model already installed and valid. Use --force to re-download."
                    );
                }
                Err(e) => {
                    eprintln!("ERROR: {}", e);
                    eprintln!("Run with --force to retry: loom install --force");
                    exit(1);
                }
            }
        }
        Some("shell") => {
            if let Err(e) = shell::run_shell().await {
                eprintln!("knowledge-loom shell failed: {e}");
                exit(1);
            }
        }
        Some("daemon") => {
            let sub_arg = args().nth(2);
            let sub = sub_arg.as_deref().unwrap_or("status");
            match sub {
                "start" => {
                    let foreground = args().any(|a| a == "--foreground");
                    if !foreground {
                        daemon::daemonize().expect("daemonize failed");
                    }
                    daemon::run_daemon_foreground()
                        .await
                        .expect("daemon failed");
                }
                "stop" => daemon::daemon_stop().expect("stop failed"),
                "status" => daemon::daemon_status(),
                "logs" => {
                    let repo = args()
                        .position(|a| a == "--repo")
                        .and_then(|i| std::env::args().nth(i + 1));
                    let follow = args().any(|a| a == "-f" || a == "--follow");
                    let n = args()
                        .position(|a| a == "-n" || a == "--lines")
                        .and_then(|i| std::env::args().nth(i + 1))
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(50usize);
                    daemon::daemon_logs(repo.as_deref(), follow, n);
                }
                "add" => {
                    let path = args().nth(3).expect("loom daemon add <path>");
                    let alias = args()
                        .position(|a| a == "--alias")
                        .and_then(|i| std::env::args().nth(i + 1));
                    daemon::add_repo(&path, alias.as_deref(), &daemon::config_path())
                        .expect("add_repo failed");
                    eprintln!("Added {path}");
                }
                "remove" => {
                    let target = args().nth(3).expect("loom daemon remove <path_or_alias>");
                    daemon::remove_repo(&target, &daemon::config_path())
                        .expect("remove_repo failed");
                    eprintln!("Removed {target}");
                }
                other => eprintln!("unknown daemon subcommand: {other}"),
            }
        }
        Some("reindex") => {
            let server =
                server::LoomServer::new(&std::env::var("KB_ROOT").expect("KB_ROOT required")).await;
            server
                .maintenance
                .reindex_all()
                .await
                .expect("reindex failed");
            eprintln!("Reindex complete.");
        }
        Some("web") => {
            let port: u16 = args()
                .position(|a| a == "--port")
                .and_then(|i| std::env::args().nth(i + 1))
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080);
            if let Err(e) = web::run_web(port).await {
                eprintln!("knowledge-loom web failed: {e}");
                exit(1);
            }
        }
        Some("serve") | None => {
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
    eprintln!("knowledge-loom — Knowledge Loom MCP server");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  loom               Start MCP stdio server (same as 'loom serve')");
    eprintln!("  loom serve         Start MCP stdio server");
    eprintln!("  loom shell         Start MCP server and open interactive shell");
    eprintln!(
        "  loom init [dir]    Initialize knowledge-loom in a directory (default: current dir)"
    );
    eprintln!(
        "  loom install       Download runtime data (fastembed models) to .knowledge-loom/models/"
    );
    eprintln!("  loom install --force  Re-download runtime data even if already installed");
    eprintln!("  loom daemon        Daemon management (start|stop|status|logs|add|remove)");
    eprintln!("  loom reindex       Reindex knowledge base (used by daemon)");
    eprintln!("  loom web [--port]  Start read-only web UI (default port 8080)");
    eprintln!("  loom help          Show this message");
    eprintln!();
    eprintln!("ENVIRONMENT:");
    eprintln!("  KB_ROOT            Root path for knowledge base (required for serve)");
}
