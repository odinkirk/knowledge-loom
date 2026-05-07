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
mod platforms;
mod shell;
mod daemon;

#[tokio::main]
async fn main() {
    match args().nth(1).as_deref() {
        Some("init")                               => {
            if let Err(e) = init::run_init(args().skip(1)) {
                eprintln!("knowledge-loom init failed: {e}");
                exit(1);
            }
        }
        Some("shell")                              => {
            if let Err(e) = shell::run_shell().await {
                eprintln!("knowledge-loom shell failed: {e}");
                exit(1);
            }
        }
        Some("daemon")                             => {
            let sub_arg = args().nth(2);
            let sub = sub_arg.as_deref().unwrap_or("status");
            match sub {
                "start" => {
                    let foreground = args().any(|a| a == "--foreground");
                    if !foreground {
                        daemon::daemonize().expect("daemonize failed");
                    }
                    daemon::run_daemon_foreground().await.expect("daemon failed");
                }
                "stop"   => daemon::daemon_stop().expect("stop failed"),
                "status" => daemon::daemon_status(),
                "logs"   => {
                    let repo = args().position(|a| a == "--repo")
                        .and_then(|i| std::env::args().nth(i + 1));
                    let follow = args().any(|a| a == "-f" || a == "--follow");
                    let n = args().position(|a| a == "-n" || a == "--lines")
                        .and_then(|i| std::env::args().nth(i + 1))
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(50usize);
                    daemon::daemon_logs(repo.as_deref(), follow, n);
                }
                "add" => {
                    let path = args().nth(3).expect("loom daemon add <path>");
                    let alias = args().position(|a| a == "--alias")
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
        Some("reindex")                            => {
            let server = server::LoomServer::new(
                &std::env::var("KB_ROOT").expect("KB_ROOT required")
            ).await;
            server.maintenance.reindex_all().await.expect("reindex failed");
            eprintln!("Reindex complete.");
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
    eprintln!("  loom shell         Start MCP server and open interactive shell");
    eprintln!("  loom init [dir]    Initialize knowledge-loom in a directory (default: current dir)");
    eprintln!("  loom daemon        Daemon management (start|stop|status|logs|add|remove)");
    eprintln!("  loom reindex       Reindex knowledge base (used by daemon)");
    eprintln!("  loom help          Show this message");
    eprintln!();
    eprintln!("ENVIRONMENT:");
    eprintln!("  KB_ROOT            Root path for knowledge base (required for serve)");
    eprintln!("  BRAINJAR_PATH      Path to brainjar binary (optional, enables loom_search_smart)");
}
