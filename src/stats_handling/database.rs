use std::{collections::HashSet, env};

use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Row, Sqlite,
};

use crate::{constants::{self}, stats_handling::device_info::Device};

/// Connects to the sqlite database and runs migrations
///
/// # Returns
/// `Pool<Sqlite>` - Interact with the database
pub async fn start_db() -> Pool<Sqlite> {
    unsafe {
        env::set_var("DATABASE_URL", "sqlite://database.sqlite");
    }

    let db_path = if cfg!(debug_assertions) {
        "database.sqlite".to_string()
    } else {
        constants::get_db_path()
    };

    let database = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(db_path)
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");



    match sqlx::migrate!("./migrations").run(&database).await {
        Ok(_) => {}
        Err(e) => eprintln!("Migration Error: {}", e),
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

    match sqlx::query_scalar::<_, String>(query)
        .bind(id)
        .fetch_optional(&*database)
        .await
    {
        // Found an entry matching this id
        Ok(_) => false,
        // Didn't find an entry matching this id
        Err(_) => true,
    }
}

/// Get all the different device uids
///
/// # Arguments
/// * `database: &Pool<Sqlite>` - Database to execute the query
///
/// # Returns
/// `HashSet<String>` - Contains all the different device uids
pub async fn get_all_device_uids(database: &Pool<Sqlite>) -> HashSet<String> {
    let mut uids = HashSet::new();

    let rows = sqlx::query("SELECT device_id FROM devices")
        .fetch_all(database)
        .await
        .expect("Failed to fetch device IDs");

    for row in rows {
        let device_id = row.get::<String, _>("device_id");

        uids.insert(device_id);
    }

    uids
}

pub async fn get_device_name_from_uid(
    database: &Pool<Sqlite>,
    device_id: impl AsRef<str>,
) -> String {
    let row = sqlx::query("SELECT device_name FROM devices WHERE device_id = ?1")
        .bind(device_id.as_ref())
        .fetch_one(&*database)
        .await
        .expect("Name not found");

    row.get("device_name")
}

pub async fn get_device_stats_after(
    database: &Pool<Sqlite>,
    device_id: &str,
    since_timestamp: i64,
) -> Vec<Device> {
    let rows = sqlx::query_as::<_, Device>(
        r#"
        SELECT *
        FROM devices
        WHERE device_id = ?1
        ORDER BY time ASC
        "#,
    )
    .bind(device_id)
    .bind(since_timestamp)
    .fetch_all(database)
    .await
    .expect("Failed to fetch device stats");

    rows
}

/// Removes all rows with the supplied device_id
/// 
/// # Arguments
/// * `database: &Pool<Sqlite>` - Database to execute the query on
/// * `device_id: String` - Device id to search for
pub async fn remove_device(database: &Pool<Sqlite>, device_id: impl AsRef<str>) {
    sqlx::query(
        r#"
        DELETE FROM devices WHERE device_id = ?1;
    "#,
    )
    .bind(device_id.as_ref())
    .execute(&*database)
    .await
    .ok();
}
