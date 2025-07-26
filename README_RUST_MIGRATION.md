# Node.js to Rust Migration Roadmap
**SD-ITS Benchmark Backend Migration: Express.js → Axum + SQLx**

## Overview
This document outlines the complete migration plan for converting the SD-ITS Benchmark backend from Node.js (Express + better-sqlite3) to Rust (Axum + SQLx).

## Current Architecture Analysis

### Existing Tech Stack
- **Framework:** Express.js 4.18.2
- **Database:** SQLite with better-sqlite3 12.2.0
- **File Upload:** Multer 2.0.2
- **Runtime:** Node.js

### Database Schema
- **8 Main Tables:** runs, performanceResult, AppDetails, SystemInfo, Libraries, GPU, RunMoreDetails, ModelMap, GPUMap, GPUBase
- **Key Features:** Foreign key relationships, indexes, transaction support
- **Database Size:** ~29MB production database

### API Endpoints
- **12 Admin Controllers:** Data processing, CRUD operations, bulk imports
- **Main Operations:** File upload processing, data transformation, bulk database operations
- **Architecture Pattern:** Controller-based with direct database access

## Target Architecture

### New Tech Stack
- **Framework:** Axum (async web framework)
- **Database:** SQLx (async SQL toolkit)
- **Runtime:** Tokio (async runtime)
- **Serialization:** Serde (JSON handling)
- **Database:** SQLite (maintaining compatibility)

---

## Migration Roadmap

### Phase 1: Project Foundation & Setup
#### 1.1 Initialize Rust Project
- [x] Create new Cargo workspace project
- [x] Set up directory structure (`src/`, `migrations/`, `tests/`)
- [x] Configure `.gitignore` for Rust project
- [x] Set up basic `Cargo.toml` with workspace configuration

#### 1.2 Dependencies Configuration
- [x] Add Axum web framework dependencies
- [x] Add SQLx with SQLite support and runtime features
- [x] Add Tokio async runtime
- [x] Add Serde for JSON serialization/deserialization
- [x] Add additional utilities:
  - [x] `uuid` for ID generation
  - [x] `chrono` for date/time handling
  - [x] `anyhow`/`thiserror` for error handling
  - [x] `tracing` for logging
  - [x] `tower` for middleware
  - [x] `tower-http` for HTTP middleware

#### 1.3 Project Structure Setup
- [x] Create modular project structure:
  ```
  src/
  ├── main.rs
  ├── lib.rs
  ├── config/
  ├── handlers/
  ├── models/
  ├── repositories/
  ├── services/
  ├── middleware/
  └── error/
  ```
- [x] Set up configuration management
- [x] Create environment configuration files

### Phase 2: Database Infrastructure
#### 2.1 Database Schema Migration
- [x] Convert existing SQLite schema to SQLx migrations
- [x] Create migration files for each table:
  - [x] `001_create_runs_table.sql`
  - [x] `002_create_performance_result_table.sql`
  - [x] `003_create_app_details_table.sql`
  - [x] `004_create_system_info_table.sql`
  - [x] `005_create_libraries_table.sql`
  - [x] `006_create_gpu_table.sql`
  - [x] `007_create_run_more_details_table.sql`
  - [x] `008_create_model_map_table.sql`
  - [x] `009_create_gpu_map_table.sql`
  - [x] `010_create_gpu_base_table.sql`
  - [x] `011_create_indexes.sql`
- [x] Set up SQLx CLI and run migrations
- [x] Verify all tables are created successfully

#### 2.2 Database Models & Types
- [x] Create Rust structs for each database table
- [x] Implement Serde traits for serialization
- [x] Add SQLx derive macros for database mapping
- [x] Create request/response DTOs
- [x] Implement type conversions and validations

#### 2.3 Database Connection & Pool
- [x] Set up SQLx connection pool configuration
- [x] Implement database initialization
- [x] Add connection health checks
- [x] Configure connection pool settings (max connections, timeouts)

### Phase 3: Core Application Infrastructure
#### 3.1 Application Bootstrap
- [x] Create main application entry point
- [x] Set up Tokio runtime configuration
- [x] Implement graceful shutdown handling
- [x] Add application state management

#### 3.2 Error Handling System
- [x] Design comprehensive error types
- [x] Implement error conversion traits
- [x] Create HTTP error responses
- [x] Add error logging and monitoring

#### 3.3 Middleware Setup
- [x] Request logging middleware
- [x] CORS middleware
- [x] Request timeout middleware
- [x] Request size limits
- [x] Security headers middleware

#### 3.4 Configuration Management
- [x] Environment-based configuration
- [x] Database configuration
- [x] Server configuration (port, host)
- [x] Logging configuration

### Phase 4: Repository Layer Implementation
#### 4.1 Base Repository Pattern
- [ ] Create generic repository traits
- [ ] Implement transaction support
- [ ] Add connection management
- [ ] Create query builder utilities

#### 4.2 Table-Specific Repositories
- [ ] `RunsRepository` - Main runs table operations
- [ ] `PerformanceResultRepository` - Performance data
- [ ] `AppDetailsRepository` - Application details
- [ ] `SystemInfoRepository` - System information
- [ ] `LibrariesRepository` - Library versions
- [ ] `GpuRepository` - GPU information
- [ ] `RunMoreDetailsRepository` - Extended run details
- [ ] `ModelMapRepository` - Model mapping
- [ ] `GpuMapRepository` - GPU mapping
- [ ] `GpuBaseRepository` - Base GPU information

### Phase 5: API Handlers Implementation
#### 5.1 File Upload Handler
- [ ] Implement multipart file upload support
- [ ] Add file validation and size limits
- [ ] Create temporary file handling
- [ ] Implement JSON parsing from uploaded files

#### 5.2 Admin API Handlers
- [ ] `/api/save-data` - Bulk data import (POST)
- [ ] `/api/process-its` - Performance data processing (POST)
- [ ] `/api/process-app-details` - App details processing (POST)
- [ ] `/api/process-system-info` - System info processing (POST)
- [ ] `/api/process-libraries` - Libraries processing (POST)
- [ ] `/api/process-gpu` - GPU data processing (POST)
- [ ] `/api/update-gpu-brands` - GPU brand updates (POST)
- [ ] `/api/update-gpu-laptop-info` - GPU laptop info (POST)
- [ ] `/api/process-run-details` - Run details processing (POST)
- [ ] `/api/app-details-analysis` - Analysis endpoint (GET)
- [ ] `/api/fix-app-names` - App name fixing (POST)
- [ ] `/api/update-run-more-details-with-modelmapid` - Model mapping (POST)

#### 5.3 Request/Response Handling
- [ ] Input validation for all endpoints
- [ ] JSON request/response serialization
- [ ] Error response formatting
- [ ] Success response standardization

### Phase 6: Business Logic Migration
#### 6.1 Data Processing Services
- [ ] Port `saveDataController` logic
- [ ] Port `processItsController` logic
- [ ] Port `processAppDetailsController` logic
- [ ] Port `processSystemInfoController` logic
- [ ] Port `processLibrariesController` logic
- [ ] Port `processGpuController` logic
- [ ] Port `updateGpuBrandsController` logic
- [ ] Port `updateGpuLaptopInfoController` logic
- [ ] Port `processRunDetailsController` logic
- [ ] Port `analyzeAppDetailsController` logic
- [ ] Port `fixAppNamesController` logic
- [ ] Port `updateRunMoreDetailsWithModelMapIdController` logic

#### 6.2 Data Parsing & Transformation
- [ ] Implement system info parsing logic
- [ ] Implement app details parsing logic
- [ ] Implement GPU data parsing logic
- [ ] Implement library version parsing logic
- [ ] Add data validation and sanitization

#### 6.3 Transaction Management
- [ ] Implement bulk insert operations with transactions
- [ ] Add rollback handling for failed operations
- [ ] Implement batch processing for large datasets
- [ ] Add progress tracking for long-running operations

### Phase 7: Testing Infrastructure
#### 7.1 Unit Testing
- [ ] Set up testing framework (cargo test)
- [ ] Create unit tests for repositories
- [ ] Create unit tests for services
- [ ] Create unit tests for data parsing logic
- [ ] Add test utilities and helpers

#### 7.2 Integration Testing
- [ ] Set up test database configuration
- [ ] Create integration tests for API endpoints
- [ ] Test file upload functionality
- [ ] Test transaction rollback scenarios
- [ ] Add end-to-end testing

#### 7.3 Performance Testing
- [ ] Benchmark bulk insert operations
- [ ] Compare performance with Node.js version
- [ ] Test concurrent request handling
- [ ] Memory usage profiling

### Phase 8: Production Readiness
#### 8.1 Security Hardening
- [ ] Input validation and sanitization
- [ ] SQL injection prevention verification
- [ ] File upload security measures
- [ ] Rate limiting implementation
- [ ] Security headers configuration

#### 8.2 Monitoring & Observability
- [ ] Structured logging implementation
- [ ] Metrics collection setup
- [ ] Health check endpoints
- [ ] Performance monitoring
- [ ] Error tracking and alerting

#### 8.3 Deployment Preparation
- [ ] Docker containerization
- [ ] Multi-stage Docker builds
- [ ] Environment variable configuration
- [ ] Database migration scripts
- [ ] Deployment documentation

#### 8.4 Documentation
- [ ] API documentation (OpenAPI/Swagger)
- [ ] Database schema documentation
- [ ] Deployment guide
- [ ] Development setup guide
- [ ] Migration guide from Node.js version

### Phase 9: Migration & Deployment
#### 9.1 Data Migration
- [ ] Export existing SQLite database
- [ ] Verify data integrity
- [ ] Import data to new Rust application
- [ ] Validate migrated data

#### 9.2 Testing & Validation
- [ ] Functional testing against production data
- [ ] Performance comparison testing
- [ ] Load testing
- [ ] User acceptance testing

#### 9.3 Deployment
- [ ] Deploy to staging environment
- [ ] Production deployment
- [ ] Monitor application performance
- [ ] Rollback plan preparation

---

## Key Migration Considerations

### Performance Expectations
- **Memory Usage:** Expect 50-70% reduction in memory usage
- **CPU Performance:** 2-3x improvement in processing speed
- **Concurrent Handling:** Better handling of concurrent requests
- **Startup Time:** Faster application startup

### Compatibility Maintenance
- **API Compatibility:** Maintain exact same REST API endpoints
- **Database Schema:** Keep existing SQLite database structure
- **Data Format:** Preserve all existing data formats and types
- **Response Format:** Maintain identical JSON response structures

### Risk Mitigation
- **Gradual Migration:** Consider running both versions in parallel initially
- **Comprehensive Testing:** Extensive testing before production deployment
- **Rollback Strategy:** Keep Node.js version available for quick rollback
- **Data Backup:** Full database backup before migration

---

## Estimated Timeline

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1-2 | 1-2 weeks | None |
| Phase 3-4 | 2-3 weeks | Phase 1-2 complete |
| Phase 5-6 | 3-4 weeks | Phase 3-4 complete |
| Phase 7 | 2-3 weeks | Phase 5-6 complete |
| Phase 8-9 | 2-3 weeks | Phase 7 complete |
| **Total** | **10-15 weeks** | |

## Success Criteria

- [ ] All existing API endpoints functional
- [ ] Performance improvements demonstrated
- [ ] All tests passing
- [ ] Production deployment successful
- [ ] Zero data loss during migration
- [ ] Monitoring and alerting operational

---

## Next Steps

1. **Review and Approve Roadmap** - Stakeholder review of migration plan
2. **Set Up Development Environment** - Rust toolchain and IDE setup
3. **Begin Phase 1** - Start with project initialization
4. **Regular Progress Reviews** - Weekly progress check-ins
5. **Risk Assessment Updates** - Continuous risk evaluation

---

## SQLx Commands Reference

### Database Management Commands

#### Installation:
```powershell
# Install SQLx CLI with SQLite support
cargo install sqlx-cli --no-default-features --features sqlite
```

#### Migration Commands:
```powershell
# Create a new migration
sqlx migrate add <migration_name>

# Run all pending migrations
sqlx migrate run

# Revert the last migration
sqlx migrate revert

# Check migration status
sqlx migrate info

# Generate migration files from existing database
sqlx migrate info --connect <database_url>
```

#### Database Commands:
```powershell
# Create database (if it doesn't exist)
sqlx database create

# Drop database
sqlx database drop

# Reset database (drop and recreate)
sqlx database reset
```

#### Development Commands:
```powershell
# Generate sqlx-data.json for offline compilation
cargo sqlx prepare

# Check SQL queries at compile time
cargo sqlx check

# Run with database connection check
cargo sqlx run
```

#### Debugging Commands:
```powershell
# Check database schema
sqlite3 my-database.db ".schema"

# List all tables
sqlite3 my-database.db ".tables"

# Execute custom SQL
sqlite3 my-database.db "SELECT * FROM runs LIMIT 5;"
```

### Common Workflows

#### Adding a New Table:
1. `sqlx migrate add create_new_table`
2. Edit the generated migration file
3. `sqlx migrate run`
4. `cargo sqlx prepare` (optional)

#### Debugging Database Issues:
1. `sqlx migrate info` - Check migration status
2. `sqlite3 my-database.db ".tables"` - List tables
3. `sqlite3 my-database.db ".schema"` - Check schema
4. `cargo check` - Verify SQL queries

#### Production Deployment:
1. `cargo sqlx prepare` - Generate offline data
2. `cargo build --release` - Build with offline support
3. `sqlx migrate run` - Run migrations on production DB

---

*This roadmap serves as a living document and should be updated as the migration progresses and new requirements or challenges are identified.* 