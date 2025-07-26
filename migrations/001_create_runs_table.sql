-- Create runs table
CREATE TABLE IF NOT EXISTS runs (
    id INTEGER PRIMARY KEY,
    timestamp TEXT,
    vram_usage TEXT,
    info TEXT,
    system_info TEXT,
    model_info TEXT,
    device_info TEXT,
    xformers TEXT,
    model_name TEXT,
    user TEXT,
    notes TEXT
);
