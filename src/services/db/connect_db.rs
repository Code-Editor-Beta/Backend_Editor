use anyhow::Result;
use mongodb::{Client, Database};
use std::env;

/**
 * function to connect_db
 */
pub async fn connect_db() -> Result<Database> {
    let uri = env::var("MONGODB_URI").expect("You must set the ENV for MongoDB_URI");

    let client = Client::with_uri_str(&uri).await?;
    let db = client.database("backendRust");
    Ok(db)
}
