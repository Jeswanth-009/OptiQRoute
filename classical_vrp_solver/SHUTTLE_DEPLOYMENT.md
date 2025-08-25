# 🚀 VRP Solver - Shuttle Deployment Guide

This document provides comprehensive instructions for migrating and deploying the VRP Solver web server to Shuttle.rs cloud platform.

## 🌟 What is Shuttle.rs?

Shuttle is a Rust-native cloud platform that makes it easy to deploy Rust applications without dealing with infrastructure. It provides:

- **Zero-config deployment**: Just add annotations to your Rust code
- **Integrated databases**: PostgreSQL, Redis, MongoDB support
- **Automatic HTTPS**: SSL certificates managed automatically
- **Git-based deployment**: Deploy from your repository
- **Free tier available**: Great for development and small projects

---

## 📋 Migration Summary

### ✅ Completed Migration Steps

1. **Updated `Cargo.toml`** - Added Shuttle dependencies:
   ```toml
   shuttle-runtime = "0.56"
   shuttle-axum = "0.56"
   ```

2. **Modified `main.rs`** - Updated to use Shuttle runtime:
   ```rust
   #[shuttle_runtime::main]
   async fn main() -> shuttle_axum::ShuttleAxum
   ```

3. **Created `Shuttle.toml`** - Configuration for deployment settings
4. **Environment handling** - Added configurable cleanup and retention settings
5. **CORS configuration** - Production-ready CORS settings

### 🔧 Key Changes Made

| Component | Original | Shuttle Version |
|-----------|----------|-----------------|
| **Main function** | `#[tokio::main]` | `#[shuttle_runtime::main]` |
| **Return type** | `Result<(), Box<dyn std::error::Error>>` | `shuttle_axum::ShuttleAxum` |
| **Server startup** | Manual `axum::serve()` | Automatic via Shuttle |
| **Configuration** | Environment variables | `Shuttle.toml` + env vars |

---

## 🚀 Deployment Instructions

### Prerequisites

1. **Install Shuttle CLI**:
   ```bash
   cargo install cargo-shuttle
   ```

2. **Create Shuttle account**:
   ```bash
   cargo shuttle login
   ```
   This will open a browser window to create/login to your account.

3. **Initialize Shuttle project**:
   ```bash
   cargo shuttle init
   ```

### Step-by-Step Deployment

#### 1. Prepare for Deployment

```bash
# Ensure your code is committed
git add .
git commit -m "Prepare for Shuttle deployment"

# Check if the project builds
cargo check --bin vrp-solver
```

#### 2. Deploy to Shuttle

```bash
# Deploy to Shuttle cloud
cargo shuttle deploy

# Or deploy with a specific name
cargo shuttle deploy --name vrp-solver-prod
```

#### 3. Monitor Deployment

```bash
# Check deployment status
cargo shuttle status

# View logs
cargo shuttle logs

# Get deployment URL
cargo shuttle status | grep "URL"
```

### Environment Variables

Set these environment variables for production deployment:

```bash
# Set via Shuttle CLI
cargo shuttle project vars set RUST_LOG=vrp_solver=info,tower_http=info
cargo shuttle project vars set CLEANUP_INTERVAL=3600
cargo shuttle project vars set DATA_RETENTION_HOURS=24
```

---

## 🔍 Version Compatibility Issues

### Current Challenge

During testing, we encountered version compatibility issues between:
- **Shuttle CLI**: v0.56.6
- **shuttle-runtime**: v0.56.0
- **Axum versions**: Shuttle uses its own Axum wrapper

### Resolution Approaches

#### Option 1: Use Compatible Versions (Recommended)
```toml
[dependencies]
shuttle-runtime = "0.47"  # Match working version
shuttle-axum = "0.47"     # Match working version
axum = "0.6"              # Compatible with Shuttle 0.47
```

#### Option 2: Alternative Deployment Platform
If Shuttle compatibility issues persist, consider:
- **Railway**: Rust-friendly platform with Dockerfile support
- **Fly.io**: Supports Rust applications with excellent documentation
- **Heroku**: Classic platform with Rust buildpack support
- **AWS Lambda**: Using `cargo-lambda` for serverless deployment

---

## 🐳 Docker Alternative Deployment

### Dockerfile for VRP Solver

Create a `Dockerfile` for containerized deployment:

```dockerfile
# Use official Rust image as builder
FROM rust:1.75 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build for release
RUN cargo build --release --bin vrp-server

# Runtime stage
FROM debian:bookworm-slim

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder stage
COPY --from=builder /app/target/release/vrp-server /usr/local/bin/vrp-server

# Set environment variables
ENV PORT=3000
ENV HOST=0.0.0.0
ENV RUST_LOG=vrp_solver=info

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl --fail http://localhost:3000/health || exit 1

# Run the application
CMD ["vrp-server"]
```

### Deploy to Various Platforms

#### Railway
```bash
# Install Railway CLI
npm install -g @railway/cli

# Login and deploy
railway login
railway init
railway up
```

#### Fly.io
```bash
# Install flyctl
curl -L https://fly.io/install.sh | sh

# Initialize and deploy
flyctl auth login
flyctl launch
flyctl deploy
```

---

## 📊 Production Configuration

### Shuttle.toml (Production Ready)
```toml
[package]
name = "vrp-solver"
version = "0.1.0"

[build]
cargo-features = []

[deploy]
assets = []

[config]
# Production settings
max_request_size = "500MB"    # Large OSM files
request_timeout = "600s"      # 10 minutes for complex VRP solving
memory_limit = "2GB"          # High memory for large datasets
cpu_limit = "1000m"           # 1 CPU core

[env]
RUST_LOG = "vrp_solver=info,tower_http=warn"
CLEANUP_INTERVAL = "1800"     # 30 minutes
DATA_RETENTION_HOURS = "12"   # 12 hours retention
```

### Performance Optimizations

1. **Memory Management**:
   ```rust
   // In main.rs - optimize for production
   let cleanup_state = app_state.clone();
   tokio::spawn(async move {
       let mut interval = tokio::time::interval(Duration::from_secs(1800)); // 30 min
       loop {
           interval.tick().await;
           if let Err(e) = cleanup_state.cleanup_old_data(12) { // 12 hours
               warn!("Cleanup failed: {}", e);
           }
       }
   });
   ```

2. **Request Limits**:
   ```rust
   // Add request size limits for production
   use tower::limit::RequestBodyLimitLayer;
   
   let app = create_routes()
       .layer(RequestBodyLimitLayer::new(500 * 1024 * 1024)) // 500MB limit
       .layer(cors)
       .with_state(app_state);
   ```

---

## 🔒 Security Considerations

### Production Security Checklist

- [ ] **CORS Configuration**: Restrict origins in production
- [ ] **Rate Limiting**: Add rate limiting middleware
- [ ] **Request Validation**: Validate all input data
- [ ] **File Upload Security**: Scan uploaded OSM files
- [ ] **Memory Limits**: Set appropriate memory limits
- [ ] **Logging**: Don't log sensitive information

### Example CORS for Production
```rust
let cors = CorsLayer::new()
    .allow_origin("https://yourdomain.com".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE])
    .max_age(Duration::from_secs(3600));
```

---

## 📈 Monitoring and Observability

### Health Check Endpoint

The VRP solver includes a comprehensive health check:
```bash
curl https://your-app.shuttleapp.rs/health
```

Response:
```json
{
  "status": "healthy",
  "timestamp": 1735083600,
  "stats": {
    "graphs": 5,
    "mappings": 3,
    "vrp_instances": 2,
    "solutions": 4
  }
}
```

### Application Metrics

Monitor these key metrics:
- **Response times**: API endpoint performance
- **Memory usage**: OSM data and solution storage
- **Success rates**: VRP solving success percentage
- **Active resources**: Number of stored graphs/solutions

### Logging Configuration

```toml
# In Shuttle.toml
[env]
RUST_LOG = "vrp_solver=info,tower_http=info,shuttle=warn"
```

---

## 🧪 Testing Deployment

### Local Testing
```bash
# Test locally before deployment
cargo shuttle run
```

### Production Testing
```bash
# After deployment, test all endpoints
curl https://your-app.shuttleapp.rs/health
curl https://your-app.shuttleapp.rs/stats

# Test file upload
curl -X POST -F "file_url=https://download.geofabrik.de/europe/monaco-latest.osm.pbf" \
     https://your-app.shuttleapp.rs/osm/upload
```

---

## 🆘 Troubleshooting

### Common Issues and Solutions

#### 1. **Shuttle Version Mismatch**
```bash
Error: shuttle-runtime 0.47.0 and Shuttle CLI 0.56.6 are incompatible
```

**Solution**: Use matching versions:
```toml
[dependencies]
shuttle-runtime = "0.47"
shuttle-axum = "0.47"
```

#### 2. **Memory Limits Exceeded**
```bash
Error: Process killed due to memory limit
```

**Solution**: Increase memory in `Shuttle.toml`:
```toml
[config]
memory_limit = "4GB"
```

#### 3. **Request Timeout**
```bash
Error: Request timeout after 30s
```

**Solution**: Increase timeout:
```toml
[config]
request_timeout = "600s"
```

#### 4. **Build Failures**
```bash
Error: Build failed due to compilation errors
```

**Solutions**:
- Check Rust version compatibility
- Update dependencies to compatible versions
- Review error logs: `cargo shuttle logs`

### Debug Commands

```bash
# View detailed logs
cargo shuttle logs --follow

# Check deployment status
cargo shuttle status --verbose

# List environment variables
cargo shuttle project vars list

# Restart deployment
cargo shuttle project restart
```

---

## 🚀 Next Steps

### Immediate Actions
1. **Resolve version conflicts**: Test with Shuttle 0.47 dependencies
2. **Deploy to staging**: Test full workflow on Shuttle
3. **Performance testing**: Load test with realistic OSM data
4. **Documentation**: Update API documentation with deployment URL

### Future Enhancements
1. **Database integration**: Add PostgreSQL for persistent storage
2. **Authentication**: Implement user authentication system
3. **Caching**: Add Redis for caching frequently used routes
4. **Monitoring**: Integrate with observability platforms
5. **CI/CD**: Automated deployment pipeline

---

## 📚 Additional Resources

### Shuttle.rs Documentation
- **Official Docs**: https://docs.shuttle.rs/
- **Examples**: https://github.com/shuttle-hq/shuttle-examples
- **Discord Community**: https://discord.gg/shuttle

### VRP Solver Resources
- **API Documentation**: `web_server.md`
- **Beginner Guide**: `beginner_guide.md`
- **Testing Results**: `webserver_prototype.md`

---

## ✅ Migration Checklist

- [x] Updated Cargo.toml with Shuttle dependencies
- [x] Modified main.rs for Shuttle runtime
- [x] Created Shuttle.toml configuration
- [x] Added environment variable handling
- [x] Updated handlers for compatibility
- [x] Created deployment documentation
- [ ] Resolved version compatibility issues
- [ ] Successfully deployed to Shuttle cloud
- [ ] Performed end-to-end testing
- [ ] Updated beginner guide with deployment URL

---

**The VRP Solver is now ready for Shuttle deployment! 🎉**

*Once version compatibility issues are resolved, the deployment should be straightforward using the provided configuration and instructions.*
