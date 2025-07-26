-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_performanceResult_run_id ON performanceResult (run_id);
CREATE INDEX IF NOT EXISTS idx_AppDetails_run_id ON AppDetails (run_id);
CREATE INDEX IF NOT EXISTS idx_SystemInfo_run_id ON SystemInfo (run_id);
CREATE INDEX IF NOT EXISTS idx_Libraries_run_id ON Libraries (run_id);
CREATE INDEX IF NOT EXISTS idx_GPU_run_id ON GPU (run_id);
CREATE INDEX IF NOT EXISTS idx_GPU_device ON GPU (device);
CREATE INDEX IF NOT EXISTS idx_RunMoreDetails_run_id ON RunMoreDetails (run_id);
CREATE INDEX IF NOT EXISTS idx_RunMoreDetails_model_name ON RunMoreDetails (model_name);
