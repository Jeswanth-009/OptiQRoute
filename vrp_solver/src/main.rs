//! VRP Solver Web Server
//! 
//! A web server that provides REST APIs for Vehicle Routing Problem solving
//! with OpenStreetMap integration.

use axum::http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use std::{env, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use vrp_solver::{
    app_state::AppState,
    handlers::create_routes,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vrp_solver=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🚛 VRP Solver Web Server Starting...");

    // Get configuration from environment variables
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    // Initialize application state
    let app_state = AppState::new();

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
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
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Run every hour
        loop {
            interval.tick().await;
            if let Err(e) = app_state.cleanup_old_data(24) { // Clean data older than 24 hours
                warn!("Failed to cleanup old data: {}", e);
            } else {
                info!("Successfully cleaned up old data");
            }
        }
    });

    // Start the server
    let listener = tokio::net::TcpListener::bind(&format!("{}:{}", host, port))
        .await
        .expect("Failed to bind to address");

    info!("🌐 Server running at http://{}:{}", host, port);
    info!("📚 API Documentation:");
    info!("  GET  /health              - Health check");
    info!("  GET  /stats               - Application statistics");
    info!("  POST /osm/upload          - Upload OSM PBF file or URL");
    info!("  POST /vrp/map             - Map depot/customers to OSM nodes");
    info!("  POST /vrp/generate        - Generate VRP instance");
    info!("  POST /vrp/solve           - Solve VRP instance");
    info!("  GET  /vrp/solution/{{id}}   - Get solution details");
    info!("  GET  /vrp/solution/{{id}}/export?format=geojson - Export solution");
    info!("");
    info!("💡 Example workflow:");
    info!("  1. Upload OSM data: curl -F 'file=@map.osm.pbf' http://{}:{}/osm/upload", host, port);
    info!("  2. Map locations: curl -X POST -H 'Content-Type: application/json' \\");
    info!("     -d '{{\"graph_id\":\"...\",\"depot\":{{\"lat\":17.735,\"lon\":83.315}},\"customers\":[...]}}' \\");
    info!("     http://{}:{}/vrp/map", host, port);
    info!("  3. Generate VRP: curl -X POST -H 'Content-Type: application/json' \\");
    info!("     -d '{{\"graph_id\":\"...\",\"vehicles\":3,\"capacity\":50,\"constraints\":{{...}}}}' \\");
    info!("     http://{}:{}/vrp/generate", host, port);
    info!("  4. Solve VRP: curl -X POST -H 'Content-Type: application/json' \\");
    info!("     -d '{{\"vrp_id\":\"...\",\"algorithm\":\"multi_start\"}}' \\");
    info!("     http://{}:{}/vrp/solve", host, port);
    info!("  5. Export solution: curl 'http://{}:{}/vrp/solution/{{id}}/export?format=geojson'", host, port);
    info!("");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");

    Ok(())
}

// Keep the original CLI functionality available as a separate binary
#[cfg(feature = "cli")]
mod cli {
    include!("cli_main.rs");
}
