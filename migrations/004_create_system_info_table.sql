-- Create SystemInfo table
CREATE TABLE IF NOT EXISTS SystemInfo (
    id INTEGER PRIMARY KEY,
    run_id INTEGER,
    arch TEXT,
    cpu TEXT,
    system TEXT,
    release TEXT,
    python TEXT,
    FOREIGN KEY (run_id) REFERENCES runs(id)
);
