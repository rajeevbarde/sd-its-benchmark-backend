-- Create performanceResult table
CREATE TABLE IF NOT EXISTS performanceResult (
    id INTEGER PRIMARY KEY,
    run_id INTEGER,
    its TEXT,
    avg_its REAL,
    FOREIGN KEY (run_id) REFERENCES runs(id)
);
