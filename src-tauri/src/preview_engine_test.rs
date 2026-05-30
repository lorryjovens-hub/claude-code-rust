use super::*;
use tokio;

#[tokio::test]
async fn test_preview_engine_default() {
    let engine = PreviewEngine::default();
    assert!(engine.contents.read().await.is_empty());
}

#[tokio::test]
async fn test_preview_engine_set_content() {
    let engine = PreviewEngine::default();
    
    let result = engine.set_content("test-id", "<html>test</html>", "text/html").await;
    assert!(result.is_ok());
    
    let contents = engine.contents.read().await;
    assert!(contents.contains_key("test-id"));
    
    let content = contents.get("test-id").unwrap();
    assert_eq!(content.id, "test-id");
    assert_eq!(content.content, "<html>test</html>");
    assert_eq!(content.content_type, "text/html");
    assert!(content.last_updated > 0);
}

#[tokio::test]
async fn test_preview_engine_get_content() {
    let engine = PreviewEngine::default();
    
    engine.set_content("test-id", "<html>test</html>", "text/html").await.unwrap();
    
    let result = engine.get_content("test-id").await;
    assert!(result.is_some());
    
    let content = result.unwrap();
    assert_eq!(content.id, "test-id");
    assert_eq!(content.content, "<html>test</html>");
}

#[tokio::test]
async fn test_preview_engine_get_nonexistent() {
    let engine = PreviewEngine::default();
    
    let result = engine.get_content("nonexistent").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_preview_engine_delete_content() {
    let engine = PreviewEngine::default();
    
    engine.set_content("test-id", "<html>test</html>", "text/html").await.unwrap();
    
    let result = engine.delete_content("test-id").await;
    assert!(result.is_ok());
    
    let contents = engine.contents.read().await;
    assert!(!contents.contains_key("test-id"));
}

#[tokio::test]
async fn test_preview_engine_list_content() {
    let engine = PreviewEngine::default();
    
    engine.set_content("id1", "<html>test1</html>", "text/html").await.unwrap();
    engine.set_content("id2", "<html>test2</html>", "text/html").await.unwrap();
    
    let contents = engine.list_contents().await;
    assert_eq!(contents.len(), 2);
}

#[tokio::test]
async fn test_preview_engine_update_content() {
    let engine = PreviewEngine::default();
    
    engine.set_content("test-id", "<html>v1</html>", "text/html").await.unwrap();
    let content1 = engine.get_content("test-id").await.unwrap();
    let time1 = content1.last_updated;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    engine.set_content("test-id", "<html>v2</html>", "text/html").await.unwrap();
    let content2 = engine.get_content("test-id").await.unwrap();
    
    assert_eq!(content2.content, "<html>v2</html>");
    assert!(content2.last_updated > time1);
}

#[tokio::test]
async fn test_preview_content_default() {
    let content = PreviewContent::default();
    assert_eq!(content.id, "");
    assert_eq!(content.content, "");
    assert_eq!(content.content_type, "");
    assert_eq!(content.last_updated, 0);
}
