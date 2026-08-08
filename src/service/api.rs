use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::{Arc, Mutex};
use serde::Deserialize;
use crate::service::{MangaEntry, query::get_manga_by_name};

use super::query::{MangaResponse, 
    build_manga_data, 
    get_manga_by_id, 
    create_response_with_manga, 
    get_all_manga_responses,
};

use super::fetch_cover_url;

pub const PRICE_LIMIT: f64 = 10_000_000.0;

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

#[derive(Deserialize)]
pub struct UpdateMangaRequest {
    pub volume_count: Option<i64>,
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

#[axum::debug_handler]
pub async fn add_manga(
    State(conn): State<SharedConn>,
    Json(payload): Json<CreateMangaRequest>,
) -> Result<Json<MangaResponse>, StatusCode> {
    let cover_url = match fetch_cover_url(&payload.name).await {
        Ok(t) => t,
        Err(_) => None,
    };

    let mut conn = conn.lock().unwrap();
    if let Ok(Some(_)) = get_manga_by_name(&conn, &payload.name) {
        return Err(StatusCode::CONFLICT);
    }

    let price = payload
        .last_price
        .unwrap_or(0.0)
        .max(0.0)
        .min(PRICE_LIMIT);

    let mut new_manga = MangaEntry::new(&payload.name, payload.volume_count, price, cover_url);
    new_manga.init(&mut conn).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match create_response_with_manga(&conn, &mut new_manga) {
        Ok(Some(manga)) => Ok(Json(manga)),
        Ok(None) => Err(axum::http::StatusCode::NOT_FOUND),
        Err(_) => Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[axum::debug_handler]
pub async fn update_manga(
    State(conn): State<SharedConn>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateMangaRequest>,
) -> Result<Json<MangaResponse>, StatusCode> {

    let mut conn = conn.lock().unwrap();
    let mut obtained_manga = match get_manga_by_id(&conn, id) {
        Ok(Some(manga)) => manga,
        Ok(None) => return Err(axum::http::StatusCode::NOT_FOUND),
        Err(_) => return Err(axum::http::StatusCode::IM_A_TEAPOT),
    };

    if let Some(mut new_price) = payload.last_price {
        if new_price >= PRICE_LIMIT {
            new_price = PRICE_LIMIT;
        }

        obtained_manga.edit_last_price(new_price);
    }

    if let Some(new_volume_count) = payload.volume_count {
        match obtained_manga.set_volume_count(&mut conn, new_volume_count) {
            Ok(_) => (),
            Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }

    match obtained_manga.save(&conn) {
        Ok(_) => (),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
    
    match create_response_with_manga(&conn, &mut obtained_manga) {
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

pub async fn remove_manga(
    State(conn): State<SharedConn>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let conn = conn.lock().unwrap();

    let mut manga = match get_manga_by_id(&conn, id) {
        Ok(Some(m)) => m,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match manga.delete(&conn) {
        Ok(_) => return Ok(StatusCode::OK),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}