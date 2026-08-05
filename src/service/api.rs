use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::{Arc, Mutex};
use serde::Deserialize;
use crate::service::MangaEntry;

use super::query::{MangaResponse, 
    build_manga_data, 
    get_manga_by_id, 
    create_response_with_manga, 
    get_all_manga_responses
};

// structures

#[derive(Deserialize)]
pub struct UpdateVolumeRequest {
    pub volume: i64,
    pub state: bool,
}

#[derive(Deserialize)]
pub struct CreateMangaRequest {
    pub name: String,
    pub volume_count: i64,
    pub last_price: Option<f64>,
}

// typedefs
type SharedConn = Arc<Mutex<rusqlite::Connection>>;

pub async fn get_manga(
    State(conn): State<SharedConn>,
    Path(id): Path<i64>,
) -> Result<Json<MangaResponse>, StatusCode> {
    let conn = conn.lock().unwrap();

    match build_manga_data(&conn, id) {
        Ok(Some(manga)) => Ok(Json(manga)),
        Ok(None) => Err(axum::http::StatusCode::NOT_FOUND),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_all_manga(State(conn): State<SharedConn>) -> Result<Json<Vec<MangaResponse>>, StatusCode> {
    let conn = conn.lock().unwrap();
    get_all_manga_responses(&conn)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn add_manga(
    State(conn): State<SharedConn>,
    Json(payload): Json<CreateMangaRequest>,
) -> Result<Json<MangaResponse>, StatusCode> {
    let mut conn = conn.lock().unwrap();
    let price: f64 = match payload.last_price {
        Some(price_value) => price_value,
        None => 0.0,
    };

    let mut new_manga = MangaEntry::new(&payload.name, payload.volume_count, price, None);
    new_manga.init(&mut conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match create_response_with_manga(&conn, &mut new_manga) {
        Ok(Some(manga)) => Ok(Json(manga)),
        Ok(None) => Err(axum::http::StatusCode::NOT_FOUND),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update_volume(
    State(conn): State<SharedConn>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateVolumeRequest>,
) -> Result<StatusCode, StatusCode> {
    let conn = conn.lock().unwrap();

    let mut manga = match get_manga_by_id(&conn, id) {
        Ok(Some(m)) => m,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let volume = manga
        .get_volume(&conn, payload.volume)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    volume
        .set_owned(&conn, payload.state)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}