-- Create Libraries table
CREATE TABLE IF NOT EXISTS Libraries (
    id INTEGER PRIMARY KEY,
    run_id INTEGER,
    torch TEXT,
    xformers TEXT,
    xformers1 TEXT,
    diffusers TEXT,
    transformers TEXT,
    FOREIGN KEY (run_id) REFERENCES runs(id)
);
