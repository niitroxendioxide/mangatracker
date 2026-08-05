pub mod manga;
pub mod query;
pub mod api;
pub mod mangadex;

pub use manga::{MangaEntry, MangaVolume};
pub use mangadex::fetch_cover_url;