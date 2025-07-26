-- Create AppDetails table
CREATE TABLE IF NOT EXISTS AppDetails (
    id INTEGER PRIMARY KEY,
    run_id INTEGER,
    app_name TEXT,
    updated TEXT,
    hash TEXT,
    url TEXT,
    FOREIGN KEY (run_id) REFERENCES runs(id)
);
