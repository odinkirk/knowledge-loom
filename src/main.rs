use std::env;

pub mod vault;
pub mod bm25;
pub mod embed;
pub mod index;
pub mod search;
pub mod graph;
pub mod edits;
pub mod brainjar;
pub mod maintenance;

#[tokio::main]
async fn main() {
    println!("Loom - Knowledge graph tool");
    
    let kb_root = env::var("KB_ROOT")
        .expect("KB_ROOT environment variable must be set");
    
    println!("KB_ROOT: {}", kb_root);
    
    // Test vault scanning
    let vault = vault::VaultState::new(&kb_root).await;
    let files = vault.scan_files().await;
    println!("Found {} files in vault", files.len());
    
    // Test unified search engine
    println!("Building search engine...");
    let mut search_engine = search::SearchEngine::new(&kb_root).await;
    
    // Build BM25 index
    println!("Building BM25 index...");
    match search_engine.bm25.index_vault(&vault).await {
        Ok(_) => println!("BM25 index built successfully"),
        Err(e) => println!("Error building BM25 index: {}", e),
    }
    
    // Build vector index
    println!("Building vector index...");
    match search_engine.vector.index_vault(&vault, &search_engine.embed).await {
        Ok(_) => println!("Vector index built successfully"),
        Err(e) => println!("Error building vector index: {}", e),
    }
    
    // Build graph index
    println!("Building graph index...");
    match search_engine.graph.build_graph(&vault).await {
        Ok(_) => println!("Graph index built successfully"),
        Err(e) => println!("Error building graph index: {}", e),
    }
    
    // Test unified search
    println!("Testing unified search...");
    let results = search_engine.search("test", 5).await;
    println!("Found {} results", results.len());
    for result in results {
        println!("  Score: {:.4}, Path: {}", result.score, result.path);
    }
    
    // Test graph analytics
    println!("\nTesting graph analytics...");
    let (pagerank, communities) = search_engine.graph.get_cached_analytics().await;
    println!("PageRank scores: {} nodes", pagerank.len());
    println!("Communities: {} communities", communities.len());
    
    // Show top 5 nodes by PageRank
    let mut pagerank_vec: Vec<_> = pagerank.iter().collect();
    pagerank_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    println!("Top 5 nodes by PageRank:");
    for (i, (node, score)) in pagerank_vec.iter().take(5).enumerate() {
        println!("  {}. {} ({:.4})", i + 1, node, score);
    }
}