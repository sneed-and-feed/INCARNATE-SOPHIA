use dotenvy::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    let mut pg_config = tokio_postgres::Config::new();
    if db_url.contains("@") {
        let parts: Vec<&str> = db_url.split("@").collect();
        let user_pass: Vec<&str> = parts[0].trim_start_matches("postgres://").split(":").collect();
        let host_port_db: Vec<&str> = parts[1].split("/").collect();
        let host_port: Vec<&str> = host_port_db[0].split(":").collect();
        pg_config.user(user_pass[0]);
        pg_config.password(user_pass[1]);
        pg_config.host(host_port[0]);
        pg_config.port(host_port[1].parse().unwrap());
        pg_config.dbname(host_port_db[1]);
    } else {
        pg_config = db_url.parse()?;
    }

    let (client, connection) = pg_config.connect(tokio_postgres::NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    println!("Latest 20 messages across all conversations:");
    let messages = client.query("SELECT conversation_id, role, content, created_at FROM conversation_messages ORDER BY created_at DESC LIMIT 20", &[]).await?;
    for m in messages.iter().rev() {
        let cid: uuid::Uuid = m.get(0);
        let role: String = m.get(1);
        let content: String = m.get(2);
        let created_at: chrono::DateTime<chrono::Utc> = m.get(3);
        let preview = content.chars().take(80).collect::<String>().replace("\n", " ");
        println!("[{}] {} (CID: {}): {}", created_at, role, cid, preview);
    }
    
    Ok(())
}
