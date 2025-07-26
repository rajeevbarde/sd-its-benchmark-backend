# 🧪 SD-ITS-Benchmark API Testing Guide

This guide provides comprehensive instructions for testing all 12 admin APIs in the SD-ITS-Benchmark Rust backend.

## 📋 Prerequisites

1. **Server Running**: Make sure the Rust server is running on port 4000
   ```bash
   cargo run
   ```

2. **Test Data**: The following files should be in your project root:
   - `test_runs_data.json` - Comprehensive test data for all APIs
   - `sample_model_map_data.sql` - Sample ModelMap data (optional)

## 🚀 Quick Start Testing

### Option 1: Automated Testing (Recommended)

#### For Linux/macOS/Windows (Git Bash):
```bash
chmod +x test_all_apis.sh
./test_all_apis.sh
```

### Option 2: Manual Testing

Follow the step-by-step manual testing instructions below.

## 📊 Test Data Overview

The `test_runs_data.json` file contains 10 test records with various scenarios:

### App Types Tested:
- **AUTOMATIC1111**: 3 records with different GPUs
- **vladmandic**: 2 records with different GPUs  
- **stable-diffusion-webui**: 2 records with different GPUs
- **null/unknown**: 3 records for edge case testing

### GPU Types Tested:
- **NVIDIA**: RTX 3080, RTX 3070 Laptop, RTX 3080 Ti, RTX 4090, RTX 3060, RTX 4080, GTX 1660 Super
- **AMD**: RX 6800 XT, RX 7900 XTX
- **Intel**: UHD Graphics 630

### System Types Tested:
- **Linux**: Various kernel versions
- **Windows**: Windows 10/11
- **macOS**: Version 13.4.1

## 🔧 Manual Testing Steps

### Step 1: Health Check
```bash
curl http://localhost:4000/health
```
**Expected**: `OK`

### Step 2: Save Data (Bulk Import)
```bash
curl -X POST \
  -H "Content-Type: multipart/form-data" \
  -F "file=@test_runs_data.json" \
  http://localhost:4000/api/save-data
```
**Expected**: JSON response with success status and insert counts

### Step 3: Process ITS (Performance Data)
```bash
curl -X POST http://localhost:4000/api/process-its
```
**Expected**: JSON response with success status and rows inserted

### Step 4: Process App Details
```bash
curl -X POST http://localhost:4000/api/process-app-details
```
**Expected**: JSON response with success status and rows inserted

### Step 5: Process System Info
```bash
curl -X POST http://localhost:4000/api/process-system-info
```
**Expected**: JSON response with success status and rows inserted

### Step 6: Process Libraries
```bash
curl -X POST http://localhost:4000/api/process-libraries
```
**Expected**: JSON response with success status and rows inserted

### Step 7: Process GPU
```bash
curl -X POST http://localhost:4000/api/process-gpu
```
**Expected**: JSON response with success status and rows inserted

### Step 8: Update GPU Brands
```bash
curl -X POST http://localhost:4000/api/update-gpu-brands
```
**Expected**: JSON response with brand counts (NVIDIA, AMD, Intel, unknown)

### Step 9: Update GPU Laptop Info
```bash
curl -X POST http://localhost:4000/api/update-gpu-laptop-info
```
**Expected**: JSON response with laptop detection counts

### Step 10: Process Run Details
```bash
curl -X POST http://localhost:4000/api/process-run-details
```
**Expected**: JSON response with success status and total inserts

### Step 11: App Details Analysis (GET)
```bash
curl http://localhost:4000/api/app-details-analysis
```
**Expected**: JSON response with analysis counts

### Step 12: Fix App Names
```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "automatic1111": "AUTOMATIC1111",
    "vladmandic": "Vladmandic", 
    "stable_diffusion": "StableDiffusion",
    "null_app_name_null_url": "Unknown"
  }' \
  http://localhost:4000/api/fix-app-names
```
**Expected**: JSON response with update counts for each category

### Step 13: Update Run More Details with ModelMapId
```bash
curl -X POST http://localhost:4000/api/update-run-more-details-with-modelmapid
```
**Expected**: JSON response with success status and update counts

## 🎯 Expected Results

### Data Processing Results:
- **Save Data**: 10 records imported
- **Process ITS**: 10 performance records created
- **Process App Details**: 10 app detail records created
- **Process System Info**: 10 system info records created
- **Process Libraries**: 10 library records created
- **Process GPU**: 10 GPU records created
- **Process Run Details**: 10 run detail records created

### GPU Brand Detection:
- **NVIDIA**: 7 GPUs (RTX 3080, RTX 3070 Laptop, RTX 3080 Ti, RTX 4090, RTX 3060, RTX 4080, GTX 1660 Super)
- **AMD**: 2 GPUs (RX 6800 XT, RX 7900 XTX)
- **Intel**: 1 GPU (UHD Graphics 630)

### GPU Laptop Detection:
- **Laptop GPUs**: 1 (RTX 3070 Laptop)
- **Desktop GPUs**: 9

### App Name Fixing:
- **AUTOMATIC1111**: 3 records updated
- **Vladmandic**: 2 records updated
- **StableDiffusion**: 2 records updated
- **Unknown**: 3 records updated (null app names)

### Model Mapping:
- **Updated**: 10 records (if ModelMap data exists)
- **Not Found**: 0 records (if all models exist in ModelMap)

## 🔍 Troubleshooting

### Common Issues:

1. **Server Not Running**
   ```
   curl: (7) Failed to connect to localhost port 4000
   ```
   **Solution**: Start the server with `cargo run`

2. **Database Errors**
   ```
   "error": "Database error: ..."
   ```
   **Solution**: Check server logs for detailed error information

3. **File Not Found**
   ```
   curl: (26) Failed to open file 'test_runs_data.json'
   ```
   **Solution**: Ensure `test_runs_data.json` is in the current directory

4. **ModelMap Data Missing**
   ```
   "message": "RunMoreDetails updated with ModelMapId successfully. Updated: 0, Not found: 10"
   ```
   **Solution**: Insert sample ModelMap data using `sample_model_map_data.sql`

### Adding ModelMap Data (Optional):
```bash
# If you want to test model mapping with actual data
sqlite3 your_database.db < sample_model_map_data.sql
```

## 📈 Performance Testing

For performance testing with larger datasets:

1. **Create larger test data**: Modify `test_runs_data.json` with more records
2. **Monitor server logs**: Watch for processing times and memory usage
3. **Database performance**: Check SQLite performance with larger datasets

## 🎉 Success Criteria

All tests are successful when:
- ✅ All 12 APIs return HTTP 200 status codes
- ✅ JSON responses contain expected data structures
- ✅ Processing counts match expected values
- ✅ No database errors in server logs
- ✅ GPU brand detection works correctly
- ✅ App name fixing updates records as expected
- ✅ Model mapping links records correctly

## 📝 Notes

- The test data includes edge cases like `NaN` values, NULL fields, and various GPU types
- All APIs are idempotent - they can be run multiple times safely
- The server uses SQLite for development/testing
- All processing is done within database transactions for data integrity 