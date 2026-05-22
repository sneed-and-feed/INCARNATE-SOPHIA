use dotenvy::dotenv;
use uuid::Uuid;

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

    println!("Cleaning up specific user prompts triggering the safety filter...");

    let cid_str = "b7ca2ee8-d99a-425d-a237-3e024230eb11";
    let cid = Uuid::parse_str(cid_str)?;

    // Delete all messages after the last safe assistant interaction to clear the bad prompt context
    let deleted = client.execute(
        "DELETE FROM conversation_messages WHERE conversation_id = $1 AND created_at > '2026-02-25 06:26:00Z'",
        &[&cid]
    ).await?;
    
    println!("Deleted {} recent messages causing the safety filter loop.", deleted);

    Ok(())
}
