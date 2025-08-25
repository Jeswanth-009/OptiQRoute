//! VRP Solver Web Server - Shuttle Deployment
//! 
//! A web server that provides REST APIs for Vehicle Routing Problem solving
//! with OpenStreetMap integration. Deployed using Shuttle.rs

use shuttle_axum::axum::http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use shuttle_axum::ShuttleAxum;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};

use vrp_solver::{
    app_state::AppState,
    handlers::create_routes,
};

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    info!("🚛 VRP Solver Web Server Starting on Shuttle...");

    // Initialize application state
    let app_state = AppState::new();

    // Setup CORS for production deployment
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .max_age(Duration::from_secs(86400)); // 24 hours

    // Create the application with middleware
    let app = create_routes()
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors)
                .into_inner(),
        )
        .with_state(app_state.clone());

    // Start cleanup task for old data
    let cleanup_state = app_state.clone();
    tokio::spawn(async move {
        // Get cleanup interval from environment (default: 1 hour)
        let cleanup_interval = std::env::var("CLEANUP_INTERVAL")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<u64>()
            .unwrap_or(3600);
        
        // Get data retention hours from environment (default: 24 hours)
        let retention_hours = std::env::var("DATA_RETENTION_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse::<u64>()
            .unwrap_or(24);
            
        info!("Cleanup task configured: interval={}s, retention={}h", cleanup_interval, retention_hours);
        
        let mut interval = tokio::time::interval(Duration::from_secs(cleanup_interval));
        loop {
            interval.tick().await;
            if let Err(e) = cleanup_state.cleanup_old_data(retention_hours) {
                warn!("Failed to cleanup old data: {}", e);
            } else {
                info!("Successfully cleaned up old data (retention: {} hours)", retention_hours);
            }
        }
    });

    info!("🌐 VRP Solver Web Server Ready!");
    info!("📚 Available API Endpoints:");
    info!("  GET  /health              - Health check and statistics");
    info!("  GET  /stats               - Application statistics");
    info!("  POST /osm/upload          - Upload OSM PBF file or URL");
    info!("  POST /vrp/map             - Map depot/customers to OSM nodes");
    info!("  POST /vrp/generate        - Generate VRP instance");
    info!("  POST /vrp/solve           - Solve VRP instance with algorithms");
    info!("  GET  /vrp/solution/{{id}}   - Get solution details");
    info!("  GET  /vrp/solution/{{id}}/export?format=geojson - Export solution");
    info!("");
    info!("💡 Ready to solve Vehicle Routing Problems with real OpenStreetMap data!");

    Ok(app.into())
}

// Keep the original CLI functionality available as a separate binary
#[cfg(feature = "cli")]
mod cli {
    include!("cli_main.rs");
}
