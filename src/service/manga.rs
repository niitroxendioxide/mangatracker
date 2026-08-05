use std::{collections::HashMap};
use rusqlite::{Connection, params, Result as sqResult};
use std::path::Path;

fn find_cover_for(name: &str) -> Option<String> {
    let slug = name.to_lowercase().replace(' ', "_");

    for ext in ["jpg", "jpeg", "png", "webp"] {
        let filename = format!("{}.{}", slug, ext);
        if Path::new("covers").join(&filename).exists() {
            return Some(filename);
        }
    }
    None
}

// Manga entry (whole identity)
pub struct MangaEntry {
    pub name: String,
    pub volume_count: i64,
    pub last_price: f64,
    pub volumes: HashMap<i64, MangaVolume>,
    pub cover_path: Option<String>,
    id: i64,
}

impl MangaEntry {
    pub fn new(name: &str, volume_count: i64, last_price: f64, cover_path: Option<String>) -> Self {
        MangaEntry {
            id: 0,
            name: name.to_owned(),
            volume_count,
            last_price,
            volumes: HashMap::new(),
            cover_path
        }
    }

    pub fn edit_last_price(&mut self, new_price: f64) {
        self.last_price = new_price;
    }

    pub fn set_volume_count(&mut self, conn: &mut Connection, new_count: i64) -> sqResult<()> {
        if new_count == self.volume_count || new_count < 1 {
            return Ok(());
        }

        let manga_id = match self.get_id(conn)? {
            Some(id) => id,
            None => return Err(rusqlite::Error::QueryReturnedNoRows),
        };

        let old_count = self.volume_count;
        let tx = conn.transaction()?;

        if new_count > old_count {
            let mut stmt = tx.prepare(
                "INSERT OR IGNORE INTO Volumes (manga_id, volume_number, owned, bought_price)
                 VALUES (?1, ?2, ?3, ?4)",
            )?;

            for vol in (old_count + 1)..=new_count {
                stmt.execute(params![manga_id, vol, false, 0.0])?;
            }
        } else {
            tx.execute(
                "DELETE FROM Volumes WHERE manga_id = ?1 AND volume_number > ?2",
                params![manga_id, new_count],
            )?;
        }

        tx.execute(
            "UPDATE Mangas SET volume_count = ?1 WHERE id = ?2",
            params![new_count, manga_id],
        )?;

        tx.commit()?;

        self.volume_count = new_count;

        if new_count < old_count {
            self.volumes.retain(|&vol_num, _| vol_num <= new_count);
        }

        Ok(())
    }

    pub fn update_metadata(&mut self, conn: &mut Connection, new_count: i64, new_price: f64) -> sqResult<()> {
        self.edit_last_price(new_price);
        
        conn.execute(
            "UPDATE Mangas SET last_price = ?1 WHERE name = ?2",
            params![self.last_price, &self.name],
        )?;

        self.set_volume_count(conn, new_count)?;

        Ok(())
    }

    pub fn save(&self, conn: &Connection) -> rusqlite::Result<()> {
        let store_query_res = self.is_stored(&conn);
        let cover_image_path = match &self.cover_path {
            Some(new_cover) => new_cover,
            None => "",
        };

        match store_query_res {
            Ok(is_stored) => {
                if is_stored {
                    conn.execute(
                        "UPDATE Mangas SET volume_count = ?2, last_price = ?3, cover_image_path=?4 WHERE name = ?1",
                        (&self.name, &self.volume_count, &self.last_price, &cover_image_path),
                    )?;
                } else {
                    conn.execute(
                        "INSERT INTO Mangas (name, volume_count, last_price, cover_image_path) VALUES (?1, ?2, ?3, ?4)",
                        (&self.name, &self.volume_count, &self.last_price, &cover_image_path),
                    )?;
                }

            }
            Err(e) => println!("{}", e)
        }

        Ok(())
    }

    pub fn set_cover(&self, conn: &Connection, new_path: &str) -> rusqlite::Result<()> {
        let id = self.id;

        conn.execute(
            "UPDATE Mangas SET cover_image_path = ?1 WHERE id = ?2",
            params![new_path, id]
        )?;
        Ok(())
    }

    pub fn get_volume(&mut self, conn: &Connection, volume_number: i64) -> sqResult<&mut MangaVolume> {
        let manga_id = match self.get_id(conn)? {
            Some(id) => id,
            None => return Err(rusqlite::Error::QueryReturnedNoRows),
        };

        if self.volumes.contains_key(&volume_number) {
            return Ok(self.volumes.get_mut(&volume_number).unwrap());
        }

        let volume = conn.query_row(
            "SELECT owned, bought_price
            FROM Volumes
            WHERE manga_id = ?1 AND volume_number = ?2",
            params![manga_id, volume_number],
            |row| {
                let owned: bool = row.get(0)?;
                let bought_price: f64 = row.get(1)?;
                Ok(MangaVolume::new(owned, volume_number, manga_id, bought_price))
            },
        )?;

        self.volumes.insert(volume_number, volume);
        let volume_instance = self.volumes.get_mut(&volume_number).unwrap();

        Ok(volume_instance)
    }

    fn is_stored(&self, conn: &Connection) -> Result<bool, rusqlite::Error> {
        match conn.query_row(
            "SELECT 1 FROM Mangas WHERE name = ?1",
            params![&self.name],
            |_| Ok(()),
        ) {
            Ok(_) => Ok(true),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn init(&mut self, conn: &mut Connection) -> sqResult<()> {
        if let Err(e) = self.save(&conn) {
            return Err(e);
        };
        
        if let Err(e) = self.initialize_volumes(conn) {
            return Err(e);
        }

        if let Err(e) = self.get_id(conn) {
            return Err(e);
        }

        if let Some(cover) = find_cover_for(&self.name) {
            if let Err(e) = self.set_cover(conn, &cover) {
                return Err(e);
            };
        }

        Ok(())
    }

    pub fn has_volumes(&self, conn: &Connection, manga_id: i64) -> sqResult<bool> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM Volumes WHERE manga_id = ?1",
            params![manga_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn initialize_volumes(&mut self, conn: &mut Connection) -> sqResult<()> {
        let manga_id = match self.get_id(conn)? {
            Some(id) => id,
            None => return Err(rusqlite::Error::QueryReturnedNoRows),
        };

        if self.has_volumes(conn, manga_id)? {
            return Ok(());
        }

        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO Volumes (manga_id, volume_number, owned, bought_price)
                VALUES (?1, ?2, ?3, ?4)",
            )?;

            for volume_number in 1..=self.volume_count {
                stmt.execute(params![manga_id, volume_number, false, 0.0 as f64])?;
            }
        }
        tx.commit()?;

        Ok(())
    }

    pub fn get_id(&mut self, conn: &Connection) -> Result<Option<i64>, rusqlite::Error> {
        if self.id > 0 {
            return Ok(Some(self.id));
        }

        match conn.query_row(
            "SELECT id FROM Mangas WHERE name = ?1",
            params![&self.name],
            |row| row.get(0),
        ) {
            Ok(id) => {
                match id {
                    Some(id_val) => {
                        self.id = id_val;
                    },
                    None => ()
                }
                Ok(id)
            },
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

// Volume entries
pub struct MangaVolume {
    pub owned: bool,
    pub volume_number: i64,
    pub manga_id: i64,
    pub bought_price: f64,

    stored_state: bool,
}

impl MangaVolume {
    pub fn new(owned: bool, volume_number: i64, manga_id: i64, bought_price: f64) -> Self {
        MangaVolume { owned, volume_number, manga_id, bought_price: bought_price, stored_state: false }
    }

    pub fn set_owned(&mut self, conn: &Connection, state: bool) -> Result<(), rusqlite::Error> {
        if self.owned == state {
            return Ok(());
        }

        self.owned = state;
        self.store(conn)?;
        Ok(())
    }

    pub fn store(&mut self, conn: &Connection) -> Result<bool, rusqlite::Error> {
        let is_stored = self.is_stored(&conn)?;

        if is_stored {
            conn.execute(
                "UPDATE Volumes SET owned=?3, bought_price=?4 WHERE manga_id=?1 AND volume_number=?2",
                params![self.manga_id, self.volume_number, self.owned, self.bought_price],
            )?;
        } else {
            self.stored_state = true;

            conn.execute(
                "INSERT INTO Volumes (manga_id, volume_number, owned, bought_price)
                 VALUES (?1, ?2, ?3, ?4)",
                params![self.manga_id, self.volume_number, self.owned, self.bought_price],
            )?;
        }

        Ok(true)
    }

    fn is_stored(&mut self, conn: &Connection) -> Result<bool, rusqlite::Error> {
        if self.stored_state == true {
            print!("Skipped stored check for true instant");
            return Ok(true);
        }

        match conn.query_row(
            "SELECT 1 FROM Volumes WHERE volume_number = ?1 AND manga_id = ?2",
            params![&self.volume_number, &self.manga_id],
            |_| Ok(()),
        ) {
            Ok(_) => {
                self.stored_state = true;
                Ok(true)
            },
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(e),
        }
    }
}