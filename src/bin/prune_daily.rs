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
    }

    let mgr = deadpool_postgres::Manager::new(pg_config, tokio_postgres::NoTls);
    let pool = deadpool_postgres::Pool::builder(mgr).max_size(1).build().unwrap();

    // Use default user_id since conversations table was wiped
    let user_id = "default".to_string();

    let workspace = ironclaw::workspace::Workspace::new(&user_id, pool);
    let path = "daily/2026-02-26.md";
    
    if let Ok(doc) = workspace.read(path).await {
        if let Some(idx) = doc.content.find("[08:15:11]") {
            let new_content = &doc.content[..idx];
            workspace.write(path, new_content).await?;
            println!("Pruned daily log from [08:15:11] onwards.");
        } else if let Some(idx) = doc.content.find("[08:15:") {
            let new_content = &doc.content[..idx];
            workspace.write(path, new_content).await?;
            println!("Pruned daily log from [08:15:] onwards.");
        } else if let Some(idx) = doc.content.find("[08:1") {
            let new_content = &doc.content[..idx];
            workspace.write(path, new_content).await?;
            println!("Pruned daily log from [08:1] onwards.");
        } else {
            println!("Could not find the timestamp in daily log.");
            println!("Last 500 chars: {}", doc.content.chars().rev().take(500).collect::<String>().chars().rev().collect::<String>());
        }
    } else {
        println!("Could not read daily/2026-02-25.md");
    }

    Ok(())
}
