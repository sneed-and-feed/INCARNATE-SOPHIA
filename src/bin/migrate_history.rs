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

    println!("Finding/Creating assistant thread for 'default'...");
    let assist_row = client.query_one(
        "SELECT id FROM conversations WHERE user_id = 'default' AND channel = 'assistant' ORDER BY last_activity DESC LIMIT 1",
        &[]
    ).await;
    
    let assistant_id = match assist_row {
        Ok(row) => row.get(0),
        Err(_) => {
            let id = uuid::Uuid::new_v4();
            client.execute(
                "INSERT INTO conversations (id, channel, user_id) VALUES ($1, 'assistant', 'default')",
                &[&id]
            ).await?;
            id
        }
    };

    println!("Consolidating all 'default' messages into assistant thread {}...", assistant_id);
    let rows = client.execute(
        "UPDATE conversation_messages SET conversation_id = $1 WHERE conversation_id IN (SELECT id FROM conversations WHERE user_id = 'default' AND id != $1)",
        &[&assistant_id]
    ).await?;
    println!("Moved {} messages", rows);
    
    println!("Cleaning up empty threads...");
    client.execute(
        "DELETE FROM conversations WHERE user_id = 'default' AND id != $1 AND id NOT IN (SELECT conversation_id FROM conversation_messages)",
        &[&assistant_id]
    ).await?;

    Ok(())
}
