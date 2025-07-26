#!/bin/bash

# Debug script to test APIs individually
BASE_URL="http://localhost:4000"

echo "🔍 Debug Testing SD-ITS-Benchmark APIs"
echo "======================================"

# Test 1: Health check
echo "1. Testing health endpoint..."
curl -s "$BASE_URL/health"
echo -e "\n"

# Test 2: Save data with minimal test
echo "2. Testing save-data with minimal data..."
cat > minimal_test.json << 'EOF'
[
  {
    "timestamp": "2024-01-01T10:00:00Z",
    "vram_usage": "8.5/12.0/15.2",
    "info": "app: test updated: 2024-01-01 hash: abc123 url: https://example.com",
    "system_info": "arch: x86_64 cpu: Intel Core i7 system: Linux release: 5.15.0 python: 3.9.7",
    "model_info": "torch: 2.0.1 xformers: 0.0.22 diffusers: 0.21.4 transformers: 4.30.2",
    "device_info": "device: NVIDIA GeForce RTX 3080 driver: 535.86.10 gpu_chip: GA102",
    "xformers": "0.0.22",
    "model_name": "test-model",
    "user": "testuser",
    "notes": "Test run"
  }
]
EOF

curl -X POST \
  -H "Content-Type: multipart/form-data" \
  -F "file=@minimal_test.json" \
  "$BASE_URL/api/save-data"
echo -e "\n"

# Test 3: Process ITS
echo "3. Testing process-its..."
curl -X POST "$BASE_URL/api/process-its"
echo -e "\n"

# Test 4: Process App Details
echo "4. Testing process-app-details..."
curl -X POST "$BASE_URL/api/process-app-details"
echo -e "\n"

# Test 5: Process System Info
echo "5. Testing process-system-info..."
curl -X POST "$BASE_URL/api/process-system-info"
echo -e "\n"

# Test 6: Process Libraries
echo "6. Testing process-libraries..."
curl -X POST "$BASE_URL/api/process-libraries"
echo -e "\n"

# Test 7: Process GPU
echo "7. Testing process-gpu..."
curl -X POST "$BASE_URL/api/process-gpu"
echo -e "\n"

# Test 8: Update GPU Brands
echo "8. Testing update-gpu-brands..."
curl -X POST "$BASE_URL/api/update-gpu-brands"
echo -e "\n"

# Test 9: Update GPU Laptop Info
echo "9. Testing update-gpu-laptop-info..."
curl -X POST "$BASE_URL/api/update-gpu-laptop-info"
echo -e "\n"

# Test 10: Process Run Details
echo "10. Testing process-run-details..."
curl -X POST "$BASE_URL/api/process-run-details"
echo -e "\n"

# Test 11: App Details Analysis
echo "11. Testing app-details-analysis..."
curl "$BASE_URL/api/app-details-analysis"
echo -e "\n"

# Test 12: Fix App Names
echo "12. Testing fix-app-names..."
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"automatic1111":"AUTOMATIC1111","vladmandic":"Vladmandic","stable_diffusion":"StableDiffusion","null_app_name_null_url":"Unknown"}' \
  "$BASE_URL/api/fix-app-names"
echo -e "\n"

# Test 13: Update Run More Details with ModelMapId
echo "13. Testing update-run-more-details-with-modelmapid..."
curl -X POST "$BASE_URL/api/update-run-more-details-with-modelmapid"
echo -e "\n"

# Cleanup
rm -f minimal_test.json

echo "🎯 Debug testing complete!" 