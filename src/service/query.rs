use rusqlite::{params, Connection, Result};
use super::{MangaEntry, MangaVolume};
use serde::Serialize;

#[derive(Serialize)]
pub struct MangaResponse {
    pub id: i64,
    pub name: String,
    pub volume_count: i64,
    pub owned_volumes: Vec<i64>,
    pub cover_image_path: Option<String>,
}

pub fn create_response_with_manga(conn: &Connection, manga: &mut MangaEntry) -> Result<Option<MangaResponse>> {
    let manga_id = match manga.get_id(&conn) {
        Ok(Some(id_val)) => id_val,
        Ok(None) => return Err(rusqlite::Error::UnwindingPanic),
        Err(e) => return Err(e)
    };

    let owned_volumes: Vec<i64> = get_owned_volumes(conn, manga_id)?
        .into_iter()
        .map(|v| v.volume_number)
        .collect();

    Ok(Some(MangaResponse {
        id: manga_id,
        name: manga.name.clone(),
        volume_count: manga.volume_count,
        owned_volumes,
        cover_image_path: manga.cover_path.clone()
    }))
}

pub fn build_manga_data(conn: &Connection, id: i64) -> Result<Option<MangaResponse>> {
    let manga = match get_manga_by_id(&conn, id)? {
        Some(m) => m,
        None => return Ok(None),
    };

    let owned_volumes: Vec<i64> = get_owned_volumes(conn, id)?
        .into_iter()
        .map(|v| v.volume_number)
        .collect();

    Ok(Some(MangaResponse {
        id: id,
        name: manga.name,
        volume_count: manga.volume_count,
        owned_volumes,
        cover_image_path: manga.cover_path.clone()
    }))
}

pub fn get_all_manga_responses(conn: &Connection) -> Result<Vec<MangaResponse>> {
    let mangas = get_all_mangas(conn)?;
    let mut responses = Vec::new();

    for mut manga in mangas {
        let id = manga.get_id(conn)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        let owned_volumes: Vec<i64> = get_owned_volumes(conn, id)?
            .into_iter()
            .map(|v| v.volume_number)
            .collect();

        responses.push(MangaResponse {
            id,
            name: manga.name,
            volume_count: manga.volume_count,
            owned_volumes,
            cover_image_path: manga.cover_path.clone()
        });
    }

    Ok(responses)
}

pub fn get_manga_by_name(conn: &Connection, name: &str) -> Result<Option<MangaEntry>> {
    let mut statement = conn.prepare("SELECT id, name, volume_count, last_price, cover_image_path FROM Mangas WHERE name = ?1")?;
    let mut rows = statement.query_map(params![name], |row| {
        let name: String = row.get(1)?;
        let volume_count: i64 = row.get(2)?;
        let last_price: f64 = row.get(3)?;
        let cover_path: Option<String> = row.get(4)?;
        
        Ok(MangaEntry::new(&name, volume_count, last_price, cover_path)) 
    })?;

    rows.next().transpose()
}

pub fn get_manga_by_id(conn: &Connection, id: i64) -> Result<Option<MangaEntry>> {
    let mut statement = conn.prepare("SELECT id, name, volume_count, last_price, cover_image_path FROM Mangas WHERE id=?1")?;
    let mut rows = statement.query_map(params![id], |row| {
        let name: String = row.get(1)?;
        let volume_count: i64 = row.get(2)?;
        let last_price: f64 = row.get(3)?;
        let cover_path: Option<String> = row.get(4)?;
        
        Ok(MangaEntry::new(&name, volume_count, last_price, cover_path)) 
    })?;

    rows.next().transpose()
}

pub fn get_all_mangas(conn: &Connection) -> Result<Vec<MangaEntry>> {
    let mut statement = conn.prepare("SELECT id, name, volume_count, last_price, cover_image_path FROM Mangas")?;
    let rows = statement.query_map((), |row| {
        let name: String = row.get(1)?;
        let volume_count: i64 = row.get(2)?;
        let last_price: f64 = row.get(3)?;
        let cover_path: Option<String> = row.get(4)?;

        Ok(MangaEntry::new(&name, volume_count, last_price, cover_path))
    })?;

    rows.collect()
}

pub fn get_owned_volumes(conn: &Connection, manga_id: i64) -> Result<Vec<MangaVolume>> {
    let mut statement = conn.prepare(
        "SELECT owned, volume_number, bought_price
         FROM Volumes
         WHERE manga_id = ?1 AND owned = 1",
    )?;
    let rows = statement.query_map(params![manga_id], |row| {
        let owned: bool = row.get(0)?;
        let volume_number: i64 = row.get(1)?;
        let bought_price: f64 = row.get(2)?;

        Ok(MangaVolume::new(owned, volume_number, manga_id, bought_price))
    })?;

    rows.collect()
}