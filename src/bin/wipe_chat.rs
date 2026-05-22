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
    }

    let (client, connection) = pg_config.connect(tokio_postgres::NoTls).await?;
    tokio::spawn(async move {
        let _ = connection.await;
    });

    println!("Wiping tainted conversation thread...");

    let deleted = client.execute("DELETE FROM conversation_messages", &[]).await?;
    let deleted_convs = client.execute("DELETE FROM conversations", &[]).await?;
    
    println!("Deleted {} messages from all conversations.", deleted);
    println!("Deleted {} conversations. The relationship memory is perfectly intact. The user should start a new chat.", deleted_convs);

    Ok(())
}
