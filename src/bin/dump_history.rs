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

    println!("--- TOTAL STATS ---");
    let total_convs: i64 = client.query_one("SELECT COUNT(*) FROM conversations", &[]).await?.get(0);
    let total_msgs: i64 = client.query_one("SELECT COUNT(*) FROM conversation_messages", &[]).await?.get(0);
    println!("Total Conversations: {}", total_convs);
    println!("Total Messages: {}", total_msgs);

    println!("--- DISTINCT USER IDS ---");
    let user_ids = client.query("SELECT DISTINCT user_id FROM conversations", &[]).await?;
    for u in user_ids {
        let uid: String = u.get(0);
        println!("  {}", uid);
    }

    println!("--- ALL CONVERSATIONS ---");
    let conv_rows = client.query("SELECT id, thread_id, user_id, channel FROM conversations ORDER BY id", &[]).await?;
    for row in conv_rows {
        let id: Uuid = row.get("id");
        let thread_id: Option<String> = row.get("thread_id");
        let user_id: String = row.get("user_id");
        let channel: String = row.get("channel");
        
        let msg_count: i64 = client.query_one("SELECT COUNT(*) FROM conversation_messages WHERE conversation_id = $1", &[&id]).await?.get(0);
        
        println!("ID: {}, Thread: {:?}, User: {}, Channel: {}, Messages: {}", id, thread_id, user_id, channel, msg_count);
        
        if msg_count > 0 {
            let messages = client.query("SELECT role, content, created_at FROM conversation_messages WHERE conversation_id = $1 ORDER BY created_at ASC", &[&id]).await?;
            for m in messages {
                let role: String = m.get("role");
                let content: String = m.get("content");
                let created_at: chrono::DateTime<chrono::Utc> = m.get("created_at");
                println!("  [{}] {}... ({})", role, content.chars().take(30).collect::<String>().replace("\n", " "), created_at);
            }
        }
    }

    println!("--- ORPHANED MESSAGES ---");
    let orphans = client.query("SELECT id, conversation_id, role, content FROM conversation_messages WHERE conversation_id NOT IN (SELECT id FROM conversations)", &[]).await?;
    println!("Found {} orphans", orphans.len());
    for o in orphans {
        let cid: Uuid = o.get("conversation_id");
        let role: String = o.get("role");
        println!("  Orphan msg for CID: {} [{}]", cid, role);
    }
    
    Ok(())
}
