use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use serde_json::{json, Value};
use std::sync::Arc;

pub struct BrainJarWrapper {
    pub path: Option<PathBuf>,
    pub process: Arc<Mutex<Option<Child>>>,
    pub request_id: Arc<Mutex<u64>>,
}

impl BrainJarWrapper {
    pub fn new(brainjar_path: Option<String>) -> Self {
        let path = brainjar_path.map(PathBuf::from);
        Self {
            path,
            process: Arc::new(Mutex::new(None)),
            request_id: Arc::new(Mutex::new(0)),
        }
    }
    
    pub async fn is_available(&self) -> bool {
        self.path.is_some() && self.path.as_ref().map(|p| p.exists()).unwrap_or(false)
    }
    
    pub async fn start(&self) -> Result<(), std::io::Error> {
        if let Some(ref path) = self.path {
            let mut process_lock = self.process.lock().await;
            
            // Check if already running
            if let Some(ref mut child) = *process_lock {
                if child.try_wait()?.is_none() {
                    return Ok(()); // Already running
                }
            }
            
            // Start new process
            let child = Command::new(path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            
            *process_lock = Some(child);
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "BrainJar path not set"))
        }
    }
    
    pub async fn call_tool(&self, tool_name: &str, args: Value) -> Result<Value, String> {
        if !self.is_available().await {
            return Err("BrainJar not available".to_string());
        }
        
        // Ensure process is running
        self.start().await.map_err(|e| e.to_string())?;
        
        let mut process_lock = self.process.lock().await;
        if let Some(ref mut child) = *process_lock {
            // Get next request ID
            let mut id_lock = self.request_id.lock().await;
            *id_lock += 1;
            let request_id = *id_lock;
            drop(id_lock);
            
            // Build MCP request
            let request = json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "tools/call",
                "params": {
                    "name": tool_name,
                    "arguments": args
                }
            });
            
            // Send request via stdin
            if let Some(stdin) = child.stdin.as_mut() {
                use tokio::io::AsyncWriteExt;
                let request_str = serde_json::to_string(&request).unwrap() + "\n";
                stdin.write_all(request_str.as_bytes()).await.map_err(|e| e.to_string())?;
            }
            
            // Read response from stdout
            if let Some(stdout) = child.stdout.as_mut() {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                
                while let Some(line) = lines.next_line().await.map_err(|e| e.to_string())? {
                    if line.trim().is_empty() {
                        continue;
                    }
                    
                    if let Ok(response) = serde_json::from_str::<Value>(&line) {
                        if response.get("id").and_then(|v| v.as_u64()) == Some(request_id) {
                            if let Some(result) = response.get("result") {
                                return Ok(result.clone());
                            } else if let Some(error) = response.get("error") {
                                return Err(error.to_string());
                            }
                        }
                    }
                }
            }
            
            Err("No response from BrainJar".to_string())
        } else {
            Err("BrainJar process not running".to_string())
        }
    }
    
    pub async fn health_check(&self) -> bool {
        if !self.is_available().await {
            return false;
        }
        
        // Try to call a simple tool or just check if process is running
        let process_lock = self.process.lock().await;
        if let Some(ref child) = *process_lock {
            // Check if process is still running
            match child.id() {
                Some(_) => true, // Process has an ID, likely running
                None => false,
            }
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub async fn stop(&self) -> Result<(), std::io::Error> {
        let mut process_lock = self.process.lock().await;
        if let Some(mut child) = process_lock.take() {
            child.kill().await?;
            child.wait().await?;
        }
        Ok(())
    }
}

impl Drop for BrainJarWrapper {
    fn drop(&mut self) {
        // Try to stop the process when wrapper is dropped
        if let Some(mut child) = self.process.try_lock().ok().and_then(|mut guard| guard.take()) {
            let _ = child.start_kill();
        }
    }
}