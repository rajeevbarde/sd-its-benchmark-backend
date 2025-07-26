-- Create GPU table
CREATE TABLE IF NOT EXISTS GPU (
    id INTEGER PRIMARY KEY,
    run_id INTEGER,
    device TEXT,
    driver TEXT,
    gpu_chip TEXT,
    brand TEXT,
    isLaptop BOOLEAN,
    FOREIGN KEY (run_id) REFERENCES runs(id)
);
