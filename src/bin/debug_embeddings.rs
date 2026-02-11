use reqwest::Client;
use serde_json::Value;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::from_path(std::path::Path::new("C:/Users/x/Desktop/ironclaw-main/.env")).ok();
    // Fallback?
    let api_key = env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY not set (checked .env in desktop folder)");
    let client = Client::new();

    println!("1. Listing Models (v1)...");
    let url = format!(
        "https://generativelanguage.googleapis.com/v1/models?key={}", // Changed to v1
        api_key
    );
    
    let resp = client.get(&url).send().await?;
    let status = resp.status();
    println!("ListModels Status: {}", status);
    
    if status.is_success() {
        let body: Value = resp.json().await?;
        if let Some(models) = body.get("models").and_then(|v| v.as_array()) {
            println!("Found {} models.", models.len());
            for m in models {
                let name = m["name"].as_str().unwrap_or("unknown");
                println!(" - {}", name);
            }
        }
    } else {
        println!("Error listing models: {}", resp.text().await?);
    }

    println!("\n2. Testing embedding-001 on v1...");
    test_embedding(&client, &api_key, "embedding-001").await;
    
    println!("\n3. Testing text-embedding-004 on v1beta...");
    test_embedding_beta(&client, &api_key, "text-embedding-004").await;

    Ok(())
}

async fn test_embedding(client: &Client, api_key: &str, model: &str) {
    // Try standard embedContent on v1
    let url = format!(
        "https://generativelanguage.googleapis.com/v1/models/{}:embedContent",
        model
    );
    
    let payload = serde_json::json!({
        "model": format!("models/{}", model),
        "content": {
            "parts": [{ "text": "Hello world" }]
        }
    });

    println!("POST {} (v1 embedContent)", url);
    let resp = client.post(&url)
        .header("x-goog-api-key", api_key)
        .json(&payload)
        .send()
        .await;

    match resp {
        Ok(r) => {
            println!("Status: {}", r.status());
            if !r.status().is_success() {
                println!("Error: {}", r.text().await.unwrap_or_default());
            } else {
                println!("Success! Embedding generated.");
            }
        }
        Err(e) => println!("Request failed: {}", e),
    }
}

async fn test_embedding_beta(client: &Client, api_key: &str, model: &str) {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:embedContent",
        model
    );
    // ... same payload ...
    let payload = serde_json::json!({
        "model": format!("models/{}", model),
        "content": {
            "parts": [{ "text": "Hello world" }]
        }
    });

    println!("POST {} (v1beta embedContent)", url);
    let resp = client.post(&url)
        .header("x-goog-api-key", api_key)
        .json(&payload)
        .send()
        .await;

     match resp {
        Ok(r) => {
            println!("Status: {}", r.status());
            if !r.status().is_success() {
                println!("Error: {}", r.text().await.unwrap_or_default());
            } else {
                println!("Success! Embedding generated.");
            }
        }
        Err(e) => println!("Request failed: {}", e),
    }
}
