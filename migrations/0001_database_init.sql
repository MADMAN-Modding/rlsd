--- Creates a table to store device information per minute
CREATE TABLE IF NOT EXISTS devices (
    id SERIAL PRIMARY KEY,
    device_id VARCHAR(255) NOT NULL,
    device_name VARCHAR(255) NOT NULL,
    ram_used BIGINT NOT NULL,
    ram_total BIGINT NOT NULL,
    cpu_usage REAL NOT NULL,
    processes INTEGER NOT NULL,
    network_in BIGINT NOT NULL,
    network_out BIGINT NOT NULL,
    time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(device_id, time)
);