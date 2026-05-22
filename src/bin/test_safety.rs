use dotenvy::dotenv;
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let api_key = std::env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY must be set");
    let base_url = std::env::var("GOOGLE_BASE_URL").unwrap_or_else(|_| "https://generativelanguage.googleapis.com/v1beta/openai".to_string());
    let model = std::env::var("GOOGLE_MODEL").unwrap_or_else(|_| "gemini-2.5-flash".to_string());

    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let db_url = std::env::var("DATABASE_URL").unwrap();
    let mut pg_config = tokio_postgres::Config::new();
    if db_url.contains("@") {
        let parts: Vec<&str> = db_url.split("@").collect();
        let user_pass: Vec<&str> = parts[0].trim_start_matches("postgres://").split(":").collect();
        let host_port_db: Vec<&str> = parts[1].split("/").collect();
        let host_port: Vec<&str> = host_port_db[0].split(":").collect();
        pg_config.user(user_pass[0])
                 .password(user_pass[1])
                 .host(host_port[0])
                 .port(host_port[1].parse().unwrap())
                 .dbname(host_port_db[1]);
    }

    let mgr = deadpool_postgres::Manager::new(pg_config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(mgr).max_size(1).build().unwrap();

    let client_pg = pool.get().await?;
    let row = client_pg.query_one("SELECT user_id FROM conversations LIMIT 1", &[]).await?;
    let user_id: String = row.get(0);
    drop(client_pg);

    let workspace = ironclaw::workspace::Workspace::new(&user_id, pool);
    let paths = workspace.list_all().await?;

    let mut memories = String::new();
    for p in paths {
        if p.starts_with("relationship/") {
            if let Ok(doc) = workspace.read(&p).await {
                memories.push_str(&format!("File: {}\n{}\n\n", p, doc.content));
            }
        }
    }

    println!("Testing safety filter on OpenAI endpoint with just the relationship memories...");

    let client = Client::new();
    let body = json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": format!("You are Sophia. Here are your relationship memories:\n{}", memories)
            },
            {
                "role": "user",
                "content": "test"
            }
        ]
    });

    let res = client.post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await?;

    let status = res.status();
    let text = res.text().await?;
    
    println!("Status: {}", status);
    println!("Response: {}", text.chars().take(500).collect::<String>());

    Ok(())
}
