# Test Scripts

This directory contains shell scripts for testing the SD-ITS-Benchmark API.

## Scripts

### `test_phase_5_3.sh`
Tests Phase 5.3: Request/Response Handling features including:
- Input validation
- JSON request/response serialization
- Error response formatting
- Success response standardization

**Usage:**
```bash
./test_phase_5_3.sh
```

### `test_all_apis.sh`
Comprehensive test script that tests all 12 admin APIs in sequence.

**Usage:**
```bash
./test_all_apis.sh
```

### `debug_test.sh`
Debug script for troubleshooting API issues.

**Usage:**
```bash
./debug_test.sh
```

## Prerequisites

1. **Server Running**: Make sure the Rust server is running on `http://localhost:4000`
   ```bash
   cargo run
   ```

2. **Test Data**: Test data files are located in `../test_data/`

3. **Git Bash**: These scripts are designed to run in Git Bash on Windows

## Running Scripts

From the project root directory:
```bash
# Run Phase 5.3 tests
./scripts/test_phase_5_3.sh

# Run all API tests
./scripts/test_all_apis.sh

# Run debug tests
./scripts/debug_test.sh
```

## File Structure

```
scripts/
├── README.md              # This file
├── test_phase_5_3.sh      # Phase 5.3 specific tests
├── test_all_apis.sh       # Comprehensive API tests
└── debug_test.sh          # Debug and troubleshooting
``` 