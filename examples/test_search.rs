use knowledge_loom::server::LoomServer;

#[tokio::main]
async fn main() {
    let kb_root = std::env::var("KB_ROOT").unwrap_or_else(|_| ".".to_string());

    println!("Searching in: {}", kb_root);
    let server = LoomServer::new(&kb_root).await;

    let query = "subdrop";
    println!("\nSearching for: '{}'", query);
    let results = server.search_engine.search(query, 10).await;

    println!("\nFound {} results:", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("\n{}. {}", i + 1, result.path);
        for s in &result.sections {
            println!(
                "   [{}] {}: {}...",
                s.chunk_ordinal,
                s.heading.as_deref().unwrap_or("no heading"),
                s.content.chars().take(200).collect::<String>()
            );
        }
    }
}
