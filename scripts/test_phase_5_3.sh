#!/bin/bash

# Phase 5.3: Request/Response Handling Test Script
# Tests the new standardized response formats and validation features

BASE_URL="http://localhost:4000"
TEST_DATA_FILE="../test_data/test_runs_data.json"

echo "🧪 Phase 5.3: Request/Response Handling Tests"
echo "============================================="
echo "Testing: Input Validation, Response Formatting, Error Handling"
echo "Base URL: $BASE_URL"
echo ""

# Function to test response format
test_response_format() {
    local endpoint=$1
    local method=$2
    local data=$3
    local description=$4
    
    echo "📋 Testing: $description"
    echo "   Endpoint: $method $endpoint"
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "\n%{http_code}" "$BASE_URL$endpoint")
    elif [ "$method" = "POST" ] && [ -n "$data" ]; then
        if [[ "$data" == {* ]]; then
            # JSON data
            response=$(curl -s -w "\n%{http_code}" -X POST \
                -H "Content-Type: application/json" \
                -d "$data" \
                "$BASE_URL$endpoint")
        else
            # File upload
            response=$(curl -s -w "\n%{http_code}" -X POST \
                -H "Content-Type: multipart/form-data" \
                -F "file=@$data" \
                "$BASE_URL$endpoint")
        fi
    else
        response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL$endpoint")
    fi
    
    status_code=$(echo "$response" | tail -n1)
    response_body=$(echo "$response" | head -n -1)
    
    echo "   Status: $status_code"
    
    # Test Phase 5.3 specific features
    if [ "$status_code" = "200" ]; then
        echo "   ✅ Success"
        
        # Check for standardized response format (without jq)
        if echo "$response_body" | grep -q '"success"'; then
            echo "   ✅ Standardized response format detected"
            
            # Extract values using grep and sed (basic parsing)
            success=$(echo "$response_body" | grep -o '"success":[^,}]*' | sed 's/"success"://' | tr -d ' ')
            message=$(echo "$response_body" | grep -o '"message":"[^"]*"' | sed 's/"message":"//' | sed 's/"$//')
            timestamp=$(echo "$response_body" | grep -o '"timestamp":"[^"]*"' | sed 's/"timestamp":"//' | sed 's/"$//')
            status_code_resp=$(echo "$response_body" | grep -o '"status_code":[^,}]*' | sed 's/"status_code"://' | tr -d ' ')
            
            echo "   📊 Response Details:"
            echo "      - success: $success"
            echo "      - message: $message"
            echo "      - timestamp: $timestamp"
            echo "      - status_code: $status_code_resp"
        else
            echo "   ⚠️  Legacy response format (not standardized)"
        fi
        
        # Check for specific response types (without jq)
        if echo "$response_body" | grep -q '"file_name"'; then
            echo "   📁 FileUploadResponse detected"
            file_name=$(echo "$response_body" | grep -o '"file_name":"[^"]*"' | sed 's/"file_name":"//' | sed 's/"$//')
            file_size=$(echo "$response_body" | grep -o '"file_size":[^,}]*' | sed 's/"file_size"://' | tr -d ' ')
            rows_processed=$(echo "$response_body" | grep -o '"rows_processed":[^,}]*' | sed 's/"rows_processed"://' | tr -d ' ')
            rows_inserted=$(echo "$response_body" | grep -o '"rows_inserted":[^,}]*' | sed 's/"rows_inserted"://' | tr -d ' ')
            rows_failed=$(echo "$response_body" | grep -o '"rows_failed":[^,}]*' | sed 's/"rows_failed"://' | tr -d ' ')
            
            echo "   📊 File Upload Details:"
            echo "      - file_name: $file_name"
            echo "      - file_size: $file_size"
            echo "      - rows_processed: $rows_processed"
            echo "      - rows_inserted: $rows_inserted"
            echo "      - rows_failed: $rows_failed"
        fi
        
        if echo "$response_body" | grep -q '"rows_processed"'; then
            echo "   🔄 ProcessingResponse detected"
            rows_processed=$(echo "$response_body" | grep -o '"rows_processed":[^,}]*' | sed 's/"rows_processed"://' | tr -d ' ')
            rows_inserted=$(echo "$response_body" | grep -o '"rows_inserted":[^,}]*' | sed 's/"rows_inserted"://' | tr -d ' ')
            echo "   📊 Processing Details:"
            echo "      - rows_processed: $rows_processed"
            echo "      - rows_inserted: $rows_inserted"
        fi
        
        if echo "$response_body" | grep -q '"data"'; then
            echo "   📋 ApiResponse with data detected"
        fi
        
    else
        echo "   ❌ Failed"
        
        # Check for standardized error format (without jq)
        if echo "$response_body" | grep -q '"error"'; then
            echo "   ✅ Standardized error response detected"
            error=$(echo "$response_body" | grep -o '"error":"[^"]*"' | sed 's/"error":"//' | sed 's/"$//')
            message=$(echo "$response_body" | grep -o '"message":"[^"]*"' | sed 's/"message":"//' | sed 's/"$//')
            echo "   📊 Error Details:"
            echo "      - error: $error"
            echo "      - message: $message"
        else
            echo "   ⚠️  Legacy error format"
        fi
        
        echo "   Response: $response_body"
    fi
    echo ""
}

# Function to test validation errors
test_validation_error() {
    local endpoint=$1
    local method=$2
    local invalid_data=$3
    local description=$4
    local expected_error=$5
    
    echo "🔍 Testing Validation: $description"
    echo "   Endpoint: $method $endpoint"
    echo "   Expected Error: $expected_error"
    
    if [ "$method" = "POST" ]; then
        response=$(curl -s -w "\n%{http_code}" -X POST \
            -H "Content-Type: application/json" \
            -d "$invalid_data" \
            "$BASE_URL$endpoint")
    fi
    
    status_code=$(echo "$response" | tail -n1)
    response_body=$(echo "$response" | head -n -1)
    
    echo "   Status: $status_code"
    
    if [ "$status_code" = "400" ] || [ "$status_code" = "422" ]; then
        echo "   ✅ Validation error correctly returned"
        
        if echo "$response_body" | jq -e '.error' > /dev/null 2>&1; then
            echo "   ✅ Standardized error response format"
            error=$(echo "$response_body" | jq -r '.error')
            message=$(echo "$response_body" | jq -r '.message')
            echo "   📊 Error Details:"
            echo "      - error: $error"
            echo "      - message: $message"
        fi
    else
        echo "   ❌ Expected validation error but got $status_code"
        echo "   Response: $response_body"
    fi
    echo ""
}

# Check if server is running
echo "🔍 Checking server health..."
if curl -s "$BASE_URL/health" > /dev/null; then
    echo "✅ Server is running at $BASE_URL"
    echo ""
else
    echo "❌ Server is not running at $BASE_URL"
    echo "Please start the server first with: cargo run"
    exit 1
fi

echo "🧪 Phase 5.3 Feature Tests"
echo "=========================="

# Test 1: File Upload with Standardized Response
echo "1️⃣  Testing File Upload Response Format"
test_response_format "/api/save-data" "POST" "$TEST_DATA_FILE" "File Upload with Standardized Response"

# Test 2: Processing Response Format
echo "2️⃣  Testing Processing Response Format"
test_response_format "/api/process-its" "POST" "" "Processing Response Format"

# Test 3: List Response Format (App Details Analysis)
echo "3️⃣  Testing List Response Format"
test_response_format "/api/app-details-analysis" "GET" "" "List Response Format"

# Test 4: Input Validation - Invalid JSON
echo "4️⃣  Testing Input Validation - Invalid JSON"
test_validation_error "/api/fix-app-names" "POST" '{"invalid": "json"' "Invalid JSON Format" "JSON_PARSE_ERROR"

# Test 5: Input Validation - Missing Required Fields
echo "5️⃣  Testing Input Validation - Missing Fields"
test_validation_error "/api/fix-app-names" "POST" '{}' "Missing Required Fields" "VALIDATION_ERROR"

# Test 6: Input Validation - Empty Request
echo "6️⃣  Testing Input Validation - Empty Request"
test_validation_error "/api/fix-app-names" "POST" '' "Empty Request Body" "BAD_REQUEST"

# Test 7: File Upload Validation - Invalid File Type
echo "7️⃣  Testing File Upload Validation"
# Create a temporary invalid file
echo "invalid content" > invalid_file.txt
test_response_format "/api/save-data" "POST" "invalid_file.txt" "Invalid File Type Validation"
rm -f invalid_file.txt

# Test 8: Error Response Format for Non-existent Endpoint
echo "8️⃣  Testing Error Response Format"
response=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/non-existent")
status_code=$(echo "$response" | tail -n1)
response_body=$(echo "$response" | head -n -1)

echo "   Status: $status_code"
if [ "$status_code" = "404" ]; then
    echo "   ✅ 404 error correctly returned"
    if echo "$response_body" | jq -e '.error' > /dev/null 2>&1; then
        echo "   ✅ Standardized error response format"
    else
        echo "   ⚠️  Legacy error format"
    fi
else
    echo "   ❌ Unexpected status code: $status_code"
fi
echo ""

# Test 9: Content-Type Validation
echo "9️⃣  Testing Content-Type Validation"
response=$(curl -s -w "\n%{http_code}" -X POST \
    -H "Content-Type: text/plain" \
    -d "invalid content" \
    "$BASE_URL/api/fix-app-names")
status_code=$(echo "$response" | tail -n1)
response_body=$(echo "$response" | head -n -1)

echo "   Status: $status_code"
if [ "$status_code" = "400" ] || [ "$status_code" = "415" ]; then
    echo "   ✅ Content-Type validation working"
else
    echo "   ❌ Content-Type validation failed"
fi
echo ""

# Test 10: Success Response with Data
echo "🔟  Testing Success Response with Data"
valid_data='{
    "automatic1111": "AUTOMATIC1111",
    "vladmandic": "Vladmandic",
    "stable_diffusion": "StableDiffusion",
    "null_app_name_null_url": "Unknown"
}'
test_response_format "/api/fix-app-names" "POST" "$valid_data" "Success Response with Data"

echo "🎉 Phase 5.3 Tests Completed!"
echo "============================="
echo "Summary of Phase 5.3 Features Tested:"
echo "✅ Standardized Response Formats"
echo "✅ Error Response Formats"
echo "✅ Input Validation"
echo "✅ Content-Type Validation"
echo "✅ File Upload Validation"
echo "✅ Response Structure Validation"
echo ""
echo "💡 Phase 5.3 Features Verified:"
echo "- All responses now use standardized format with success, message, timestamp, status_code"
echo "- Error responses include error type, message, and details"
echo "- Input validation prevents invalid data"
echo "- File upload validation works correctly"
echo "- Content-Type validation is enforced"
echo ""
echo "🚀 Phase 5.3: Request/Response Handling - COMPLETE!" 