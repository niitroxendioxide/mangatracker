use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct SearchResponse {
    pub data: Vec<MangaData>,
}

#[derive(Deserialize, Debug)]
pub struct MangaData {
    pub id: String,
    pub r#type: String,
    pub attributes: MangaAttributes,
    #[serde(default)]
    pub relationships: Vec<Relationship>,
}

#[derive(Deserialize, Debug)]
pub struct MangaAttributes {
    pub title: HashMap<String, String>,
    #[serde(default)]
    pub alt_titles: Vec<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
pub struct Relationship {
    pub id: String,
    #[serde(rename = "type")]
    pub rel_type: String,
    pub attributes: Option<CoverAttributes>,
}

#[derive(Deserialize, Debug)]
pub struct CoverAttributes {
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
}

pub async fn fetch_cover_url(manga_name: &str) -> Result<Option<String>, reqwest::Error> {
    let client = reqwest::Client::builder()
        .user_agent("MyMangaApp/1.0")
        .build()?;

    let query_params = [
        ("title", manga_name),
        ("limit", "20"),
        ("includes[]", "cover_art"),
        ("order[followedCount]", "desc"),
    ];

    let search: SearchResponse = client
        .get("https://api.mangadex.org/manga")
        .query(&query_params)
        .send()
        .await?
        .json()
        .await?;

    let clean_target = manga_name.trim();

    let is_exact_match = |title_map: &HashMap<String, String>| {
        title_map.values().any(|val| {
            let t = val.trim();
            t.eq_ignore_ascii_case(clean_target)
        })
    };

    let target_manga = search.data.iter().find(|manga| {
        if is_exact_match(&manga.attributes.title) {
            return true;
        }
        manga.attributes.alt_titles.iter().any(is_exact_match)
    })
    .or_else(|| {
        search.data.iter().find(|manga| {
            manga.attributes.title.values().chain(
                manga.attributes.alt_titles.iter().flat_map(|m| m.values())
            ).any(|t| {
                t.split(|c: char| !c.is_alphanumeric())
                 .any(|word| word.eq_ignore_ascii_case(clean_target))
            })
        })
    });

    let Some(target_manga) = target_manga else {
        return Ok(None);
    };

    let file_name = target_manga
        .relationships
        .iter()
        .find(|r| r.rel_type == "cover_art")
        .and_then(|r| r.attributes.as_ref())
        .and_then(|attr| attr.file_name.as_ref());

    match file_name {
        Some(name) => Ok(Some(format!(
            "https://uploads.mangadex.org/covers/{}/{}",
            target_manga.id, name
        ))),
        None => Ok(None),
    }
}