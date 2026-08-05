use rusqlite::{Connection, Result};
// use super::data::MangaEntry;

pub fn init_db() -> Result<Connection> {
    let conn = Connection::open("mangatracker.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Mangas (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            name         TEXT NOT NULL,
            volume_count INTEGER NOT NULL,
            cover_image_path TEXT,
            last_price DOUBLE PRECISION NOT NULL
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS Volumes (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            manga_id         INTEGER NOT NULL,
            volume_number INTEGER NOT NULL,
            owned BOOLEAN NOT NULL,
            bought_price DOUBLE PRECISION
        )",
        (),
    )?;

    Ok(conn)
}