-- Create GPUMap table
CREATE TABLE IF NOT EXISTS GPUMap (
    id INTEGER PRIMARY KEY,
    gpu_name TEXT,
    base_gpu_id INTEGER REFERENCES GPUBase(id)
);
