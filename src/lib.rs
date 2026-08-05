pub mod service;
pub mod db;

use rusqlite::Connection;
use service::{MangaEntry};

pub fn seed_db(conn: &mut Connection) -> rusqlite::Result<()> {
    let mut new_manga = MangaEntry::new("Kagurabachi", 11, 0.0, None);
    new_manga.init(conn)?;

    let mut new_manga = MangaEntry::new("Nana", 21, 11000.0, None);
    new_manga.init(conn)?;

    let mut new_manga = MangaEntry::new("Death Note", 12, 10500.0, None);
    new_manga.init(conn)?;

    let mut new_manga = MangaEntry::new("Fullmetal Alchemist", 27, 11000.0, None);
    new_manga.init(conn)?;

    let mut new_manga = MangaEntry::new("Gokinjo Monogatari", 5, 0.0, None);
    new_manga.init(conn)?;

    let mut new_manga = MangaEntry::new("Jigokuraku", 13, 0.0, None);
    new_manga.init(conn)?;

    Ok(())
}