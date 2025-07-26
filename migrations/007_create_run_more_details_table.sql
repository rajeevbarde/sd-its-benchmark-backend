-- Create RunMoreDetails table
CREATE TABLE IF NOT EXISTS RunMoreDetails (
    id INTEGER PRIMARY KEY,
    run_id INTEGER,
    timestamp TEXT,
    model_name TEXT,
    user TEXT,
    notes TEXT,
    ModelMapId INTEGER,
    FOREIGN KEY (run_id) REFERENCES runs(id)
);
