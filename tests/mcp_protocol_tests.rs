use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::process::{Command, Stdio};

#[tokio::test]
async fn test_mcp_protocol_initialization() {
    let temp_dir = tempfile::tempdir().unwrap();
    let kb_root = temp_dir.path().to_str().unwrap();

    // Create a simple test file
    std::fs::write(temp_dir.path().join("test.md"), "# Test\nContent").unwrap();

    // Start the MCP server
    let mut child = Command::new(
        &std::env::var("CARGO_BIN_EXE_loom")
            .unwrap_or_else(|_| format!("{}/target/debug/loom", env!("CARGO_MANIFEST_DIR"))),
    )
    .arg("serve")
    .env("KB_ROOT", kb_root)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("Failed to start loom server");

    // Get stdin and stdout immediately
    let stdin = child.stdin.as_mut().expect("Failed to get stdin");
    let stdout = child.stdout.as_mut().expect("Failed to get stdout");
    let stderr = child.stderr.as_mut().expect("Failed to get stderr");

    // Send initialize request IMMEDIATELY (rmcp expects this)
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    writeln!(stdin, "{}", init_request).expect("Failed to write to stdin");
    stdin.flush().expect("Failed to flush stdin");

    // Read response
    let reader = std::io::BufReader::new(stdout);
    let mut lines = reader.lines();

    if let Some(Ok(line)) = lines.next() {
        let response: Value = serde_json::from_str(&line).expect("Failed to parse response");
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 1);
        assert!(response["result"].is_object());
        println!(
            "Initialize response: {}",
            serde_json::to_string_pretty(&response).unwrap()
        );
    } else {
        // Check stderr for errors
        let mut stderr_lines = std::io::BufReader::new(stderr).lines();
        if let Some(Ok(stderr_line)) = stderr_lines.next() {
            eprintln!("Server stderr: {}", stderr_line);
        }
        panic!("No response from server");
    }

    // Send initialized notification IMMEDIATELY
    let initialized = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });

    writeln!(stdin, "{}", initialized).expect("Failed to write initialized");
    stdin.flush().expect("Failed to flush initialized");

    // Give the server time to process
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Send tools/list request
    let tools_list = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });

    writeln!(stdin, "{}", tools_list).expect("Failed to write tools/list");
    stdin.flush().expect("Failed to flush stdin");

    // Read tools response with timeout
    let start = std::time::Instant::now();
    let mut found_response = false;

    while start.elapsed() < std::time::Duration::from_secs(5) {
        if let Some(Ok(line)) = lines.next() {
            println!("Received: {}", line);
            let response: Value =
                serde_json::from_str(&line).expect("Failed to parse tools response");

            if response["id"] == 2 {
                found_response = true;
                assert_eq!(response["jsonrpc"], "2.0");

                if let Some(result) = response.get("result") {
                    if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
                        println!("Found {} tools", tools.len());
                        assert!(!tools.is_empty(), "Should have at least one tool");

                        // Check for expected tools
                        let tool_names: Vec<&str> = tools
                            .iter()
                            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
                            .collect();

                        assert!(tool_names.contains(&"search"), "Should have search tool");
                        assert!(
                            tool_names.contains(&"list_files"),
                            "Should have list_files tool"
                        );
                    } else {
                        panic!("No tools array in response");
                    }
                } else {
                    panic!("No result in tools/list response");
                }
                break;
            }
        }
    }

    if !found_response {
        panic!("No tools response from server within timeout");
    }

    // Kill the server
    child.kill().expect("Failed to kill server");
}

#[tokio::test]
async fn test_mcp_tool_call() {
    let temp_dir = tempfile::tempdir().unwrap();
    let kb_root = temp_dir.path().to_str().unwrap();

    // Create a simple test file
    std::fs::write(temp_dir.path().join("test.md"), "# Test\nContent").unwrap();

    // Start the MCP server
    let mut child = Command::new(
        &std::env::var("CARGO_BIN_EXE_loom")
            .unwrap_or_else(|_| format!("{}/target/debug/loom", env!("CARGO_MANIFEST_DIR"))),
    )
    .arg("serve")
    .env("KB_ROOT", kb_root)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("Failed to start loom server");

    let stdin = child.stdin.as_mut().expect("Failed to get stdin");
    let stdout = child.stdout.as_mut().expect("Failed to get stdout");
    let reader = std::io::BufReader::new(stdout);
    let mut lines = reader.lines();

    // Send initialize request immediately
    writeln!(
        stdin,
        "{}",
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        })
    )
    .unwrap();
    stdin.flush().expect("Failed to flush stdin");

    lines.next(); // Skip init response

    // Send initialized
    writeln!(
        stdin,
        "{}",
        json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        })
    )
    .unwrap();
    stdin.flush().expect("Failed to flush stdin");

    // Call list_files
    writeln!(
        stdin,
        "{}",
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "list_files",
                "arguments": {}
            }
        })
    )
    .unwrap();
    stdin.flush().expect("Failed to flush stdin");

    // Read response with timeout
    let start = std::time::Instant::now();
    let mut found_response = false;

    while start.elapsed() < std::time::Duration::from_secs(5) {
        if let Some(Ok(line)) = lines.next() {
            println!("Received: {}", line);
            let response: Value = serde_json::from_str(&line).expect("Failed to parse response");

            if response["id"] == 2 {
                found_response = true;
                assert_eq!(response["jsonrpc"], "2.0");

                if let Some(result) = response.get("result") {
                    if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
                        assert!(!content.is_empty(), "Should have content");
                        println!(
                            "Tool call result: {}",
                            serde_json::to_string_pretty(&result).unwrap()
                        );
                    } else {
                        panic!("No content in tool result");
                    }
                } else {
                    let error = response.get("error").expect("Tool call failed");
                    panic!("Tool call error: {}", error);
                }
                break;
            }
        }
    }

    if !found_response {
        panic!("No response from tool call within timeout");
    }

    child.kill().expect("Failed to kill server");
}
