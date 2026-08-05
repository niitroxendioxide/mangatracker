// Dependencies/libraries loaded

use tower_http::services::ServeFile;
use tower_http::{cors::CorsLayer, services::ServeDir};
use axum::{routing::get, routing::post,Router,};
use std::sync::{Arc, Mutex};
use mangatracker::db;
use mangatracker::service::api;
use mangatracker::seed_db;

type SharedConn = Arc<Mutex<rusqlite::Connection>>;


// Main logic
#[tokio::main]
async fn main() {
    let mut conn = db::init_db().expect("failed to init db");
    if let Err(e) = seed_db(&mut conn) {
        println!("Couldn\'t seed the database. {}", e);
    };

    let shared_conn: SharedConn = Arc::new(Mutex::new(conn));

    let app = Router::new()
    .route("/manga/{id}", get(api::get_manga))
    .route("/manga/create", post(api::add_manga))
    .route("/manga/update/{id}", post(api::update_volume))
    .route("/manga", get(api::get_all_manga))
    .nest_service("/covers", ServeDir::new("covers"))
    .route_service("/", ServeFile::new("frontend/index.html"))
    .route_service("/app.js", ServeFile::new("frontend/app.js"))
    .layer(CorsLayer::permissive())
    .with_state(shared_conn);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://0.0.0.0:3000");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap()
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for [CTRL + C]");
}