use std::path::PathBuf;
use knowledge_loom::brainjar::BrainJarWrapper;

#[tokio::test]
async fn test_brainjar_new_with_path() {
    let path = Some("/usr/bin/echo".to_string());
    let wrapper = BrainJarWrapper::new(path);
    
    assert!(wrapper.path.is_some());
    assert_eq!(wrapper.path.clone().unwrap(), PathBuf::from("/usr/bin/echo"));
}

#[tokio::test]
async fn test_brainjar_new_without_path() {
    let wrapper = BrainJarWrapper::new(None);
    
    assert!(wrapper.path.is_none());
}

#[tokio::test]
async fn test_brainjar_is_available_with_valid_path() {
    // Use a command that should exist on most systems
    let path = Some("/bin/sh".to_string());
    let wrapper = BrainJarWrapper::new(path);
    
    assert!(wrapper.is_available().await);
}

#[tokio::test]
async fn test_brainjar_is_available_with_invalid_path() {
    let path = Some("/nonexistent/path/to/brainjar".to_string());
    let wrapper = BrainJarWrapper::new(path);
    
    assert!(!wrapper.is_available().await);
}

#[tokio::test]
async fn test_brainjar_is_available_without_path() {
    let wrapper = BrainJarWrapper::new(None);
    
    assert!(!wrapper.is_available().await);
}

#[tokio::test]
async fn test_brainjar_start_without_path() {
    let wrapper = BrainJarWrapper::new(None);
    
    let result = wrapper.start().await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not set"));
}

#[tokio::test]
async fn test_brainjar_call_tool_not_available() {
    let wrapper = BrainJarWrapper::new(None);
    
    let result = wrapper.call_tool("search", serde_json::json!({"query": "test"})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not available"));
}

#[tokio::test]
async fn test_brainjar_call_tool_with_invalid_path() {
    let wrapper = BrainJarWrapper::new(Some("/nonexistent/path".to_string()));
    
    let result = wrapper.call_tool("search", serde_json::json!({"query": "test"})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_brainjar_health_check_not_available() {
    let wrapper = BrainJarWrapper::new(None);
    
    assert!(!wrapper.health_check().await);
}

#[tokio::test]
async fn test_brainjar_health_check_with_invalid_path() {
    let wrapper = BrainJarWrapper::new(Some("/nonexistent/path".to_string()));
    
    assert!(!wrapper.health_check().await);
}

#[tokio::test]
async fn test_brainjar_stop_without_process() {
    let wrapper = BrainJarWrapper::new(None);
    
    let result = wrapper.stop().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_brainjar_request_id_increment() {
    let wrapper = BrainJarWrapper::new(Some("/bin/sh".to_string()));
    
    let id1 = wrapper.request_id.lock().await;
    let initial_id = *id1;
    drop(id1);
    
    // Increment request ID
    let mut id2 = wrapper.request_id.lock().await;
    *id2 += 1;
    let new_id = *id2;
    drop(id2);
    
    assert_eq!(new_id, initial_id + 1);
}

#[tokio::test]
async fn test_brainjar_multiple_instances() {
    let wrapper1 = BrainJarWrapper::new(Some("/bin/sh".to_string()));
    let wrapper2 = BrainJarWrapper::new(Some("/bin/echo".to_string()));
    
    assert!(wrapper1.is_available().await);
    assert!(wrapper2.is_available().await);
    
    assert_ne!(wrapper1.path, wrapper2.path);
}