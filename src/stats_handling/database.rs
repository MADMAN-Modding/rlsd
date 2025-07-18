use std::env;

use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, Pool, Sqlite};

use crate::stats_handling::device_info::Device;

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

/// Inserts data into the database
/// 
/// # Arguments
/// * `device: Device` - Struct to insert
/// * `database: &Pool<Sqlite>` - Database to use to execute
/// 
/// # Returns
/// * `Ok()` - Insertion succeeds
/// * `Err(sqlx::Error)` - Insertion fails 
pub async fn input_data(device: Device, database: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO devices (device_id, device_name, ram_used, ram_total, cpu_usage, processes, network_in, network_out, time)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#
    )
    .bind(device.device_id)
    .bind(device.device_name)
    .bind(device.ram_used)
    .bind(device.ram_total)
    .bind(device.cpu_usage)
    .bind(device.processes)
    .bind(device.network_in)
    .bind(device.network_out)
    .bind(device.time).execute(&*database)
    .await?;

    Ok(())
}

pub async fn check_device_id_exists(id: &String, database: &Pool<Sqlite>) -> bool {
    let query = r#"
        SELECT * FROM devices
        WHERE id = ?1
        ORDER BY RANDOM()
        LIMIT 1
    "#;

    match sqlx::query_scalar::<_, String> (
        query
    )
    .bind(id)
    .fetch_optional(&*database).await {
        // Found an entry matching this id
        Ok(_) => false,
        // Didn't find an entry matching this id
        Err(_) => true
    }
}