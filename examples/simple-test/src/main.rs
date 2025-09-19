use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct TestMessage {
    id: u32,
    content: String,
    metadata: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Simple Test Application Starting...");
    
    let message = TestMessage {
        id: 1,
        content: "Hello from minimal Docker container!".to_string(),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("source".to_string(), "simple-test".to_string());
            meta.insert("timestamp".to_string(), chrono::Utc::now().to_rfc3339());
            meta
        },
    };
    
    let json = serde_json::to_string_pretty(&message)?;
    println!("📦 Test Message:");
    println!("{}", json);
    
    println!("✅ Simple test completed successfully!");
    Ok(())
}
