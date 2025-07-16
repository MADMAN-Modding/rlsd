use std::env;

use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, Pool, Sqlite};

/// Connects to the sqlite database and runs migrations
/// 
/// # Returns
/// `Pool<Sqlite>` - Interact with the database
pub async fn start_db() -> Pool<Sqlite> {
    unsafe {
        env::set_var("DATABASE_URL", "sqlite://database.sqlite");
    }

    let database = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            SqliteConnectOptions::new()
                .filename("database.sqlite")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");
    
    match sqlx::migrate!("./migrations")
        .run(&database)
        .await {
            Ok(_) => {},
            Err(e) => eprintln!("Migration Error: {}", e)
        };

    database
}
