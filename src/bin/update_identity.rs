use ironclaw::config::Config;
use ironclaw::history::Store;
use ironclaw::workspace::Workspace;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let _ = dotenvy::dotenv(); // Load environment variables just in case
    let config = Config::from_env()?;
    let store = Store::new(&config.database).await?;
    let pool = store.pool();
    
    // Default user ID used in gateway
    let user_id = "local_user";
    
    let db_workspace = Workspace::new(user_id, pool);

    println!("Forcing identity seed push...");
    
    let identity_files = [
        "AGENTS.md",
        "SOUL.md",
        "IDENTITY.md",
        "USER.md",
    ];

    for path in identity_files {
        if let Ok(content) = tokio::fs::read_to_string(format!("templates/{}", path)).await {
            println!("Overwriting DB workspace with: {}", path);
            db_workspace.write(path, &content).await?;
        }
    }
    
    println!("Done! DB memory overlay updated.");
    Ok(())
}
