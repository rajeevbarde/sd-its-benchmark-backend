#!/bin/bash

# Comprehensive API Test Script for SD-ITS-Benchmark
# Tests all 12 admin APIs in sequence

BASE_URL="http://localhost:4000"
TEST_DATA_FILE="test_runs_data.json"

echo "🚀 Starting Comprehensive API Tests for SD-ITS-Benchmark"
echo "=================================================="
echo "Base URL: $BASE_URL"
echo "Test Data: $TEST_DATA_FILE"
echo ""

# Function to make API calls and display results
make_api_call() {
    local method=$1
    local endpoint=$2
    local data_file=$3
    local description=$4
    
    echo "📋 Testing: $description"
    echo "   Endpoint: $method $endpoint"
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "\n%{http_code}" "$BASE_URL$endpoint")
    elif [ "$method" = "POST" ] && [ -n "$data_file" ]; then
        # Check if data_file is a JSON string (starts with {)
        if [[ "$data_file" == {* ]]; then
            response=$(curl -s -w "\n%{http_code}" -X POST \
                -H "Content-Type: application/json" \
                -d "$data_file" \
                "$BASE_URL$endpoint")
        else
            response=$(curl -s -w "\n%{http_code}" -X POST \
                -H "Content-Type: multipart/form-data" \
                -F "file=@$data_file" \
                "$BASE_URL$endpoint")
        fi
    else
        response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL$endpoint")
    fi
    
    # Extract status code (last line)
    status_code=$(echo "$response" | tail -n1)
    # Extract response body (all lines except last)
    response_body=$(echo "$response" | head -n -1)
    
    echo "   Status: $status_code"
    if [ "$status_code" = "200" ]; then
        echo "   ✅ Success"
        echo "   Response: $response_body" | jq '.' 2>/dev/null || echo "   Response: $response_body"
    else
        echo "   ❌ Failed"
        echo "   Response: $response_body"
    fi
    echo ""
}

# Check if server is running
echo "🔍 Checking if server is running..."
if curl -s "$BASE_URL/health" > /dev/null; then
    echo "✅ Server is running at $BASE_URL"
    echo ""
else
    echo "❌ Server is not running at $BASE_URL"
    echo "Please start the server first with: cargo run"
    exit 1
fi

# Test 1: Save Data (Bulk Import)
echo "1️⃣  Testing Data Import"
make_api_call "POST" "/api/save-data" "$TEST_DATA_FILE" "Save Data - Bulk Import"

# Test 2: Process ITS (Performance Data)
echo "2️⃣  Testing Performance Data Processing"
make_api_call "POST" "/api/process-its" "" "Process ITS - Performance Data"

# Test 3: Process App Details
echo "3️⃣  Testing App Details Processing"
make_api_call "POST" "/api/process-app-details" "" "Process App Details"

# Test 4: Process System Info
echo "4️⃣  Testing System Info Processing"
make_api_call "POST" "/api/process-system-info" "" "Process System Info"

# Test 5: Process Libraries
echo "5️⃣  Testing Libraries Processing"
make_api_call "POST" "/api/process-libraries" "" "Process Libraries"

# Test 6: Process GPU
echo "6️⃣  Testing GPU Processing"
make_api_call "POST" "/api/process-gpu" "" "Process GPU"

# Test 7: Update GPU Brands
echo "7️⃣  Testing GPU Brand Updates"
make_api_call "POST" "/api/update-gpu-brands" "" "Update GPU Brands"

# Test 8: Update GPU Laptop Info
echo "8️⃣  Testing GPU Laptop Info Updates"
make_api_call "POST" "/api/update-gpu-laptop-info" "" "Update GPU Laptop Info"

# Test 9: Process Run Details
echo "9️⃣  Testing Run Details Processing"
make_api_call "POST" "/api/process-run-details" "" "Process Run Details"

# Test 10: App Details Analysis (GET)
echo "🔟  Testing App Details Analysis"
make_api_call "GET" "/api/app-details-analysis" "" "App Details Analysis"

# Test 11: Fix App Names
echo "1️⃣1️⃣  Testing App Name Fixing"
fix_app_names_data='{
    "automatic1111": "AUTOMATIC1111",
    "vladmandic": "Vladmandic",
    "stable_diffusion": "StableDiffusion",
    "null_app_name_null_url": "Unknown"
}'
make_api_call "POST" "/api/fix-app-names" "$fix_app_names_data" "Fix App Names"

# Test 12: Update Run More Details with ModelMapId
echo "1️⃣2️⃣  Testing Model Mapping"
make_api_call "POST" "/api/update-run-more-details-with-modelmapid" "" "Update Run More Details with ModelMapId"

echo "🎉 All API Tests Completed!"
echo "=================================================="
echo "Summary:"
echo "- 12 APIs tested"
echo "- Check the responses above for any errors"
echo "- All endpoints should return 200 status codes"
echo ""
echo "💡 Tips:"
echo "- If any API fails, check the server logs for details"
echo "- The test data includes various scenarios (AUTOMATIC1111, vladmandic, etc.)"
echo "- GPU brand detection should work for NVIDIA, AMD, and Intel GPUs"
echo "- App name fixing should update records based on URL patterns"
echo "- Run 'cargo test' to run unit tests" 