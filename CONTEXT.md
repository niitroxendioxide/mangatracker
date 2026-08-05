# Colección de Manga — Project Idea

## 1. Concept
A small self-hosted app to track a personal manga collection. Opens to a main
menu of manga series shown as book-cover icons. Tapping a cover opens a detail
tab showing which volumes are owned and which are missing.

Accessed from phones (iPhone 15 Plus, iPhone 17) over the home network (and
optionally remotely), served from a mini PC running 24/7.

## 2. Screens

### Home / Menu
- Title: **"Colección de Manga"**
- Grid of manga covers, one icon per series
- Icons are **rectangles taller than they are wide** (book/volume proportions,
  roughly a 2:3 or 3:4 aspect ratio)
- Tapping an icon navigates to that series' detail tab

### Series Detail
- Shows the series name/cover
- List or grid of volumes (e.g. 1–30)
- Visual distinction between **owned** and **missing** volumes
  (e.g. color, checkmark, grayscale for missing)

## 3. Data Model (draft)

```
Manga
- id
- title
- cover_image_path
- total_volumes (optional, can be "unknown/ongoing")

Volume
- id
- manga_id (FK)
- number
- owned (bool)
```

Could later extend with: read/unread status, purchase links, price paid,
notes, tags/genre.

## 4. Tech Stack (proposed)

**Backend**
- [Axum](https://github.com/tokio-rs/axum) (Tokio-based) for the HTTP API
- [SQLx](https://github.com/launchbadge/sqlx) or `rusqlite` + SQLite as the
  database (simple, file-based, fine for a personal collection)
- `serde` / `serde_json` for API payloads
- `tower-http` for static file serving, CORS, compression

**Frontend**
- Plain **HTML + CSS + vanilla JavaScript** — no framework (no React, no
  Rust/WASM). Kept deliberately simple: a grid of book-shaped cards, a click
  handler to swap to the detail view, `fetch()` calls to the Rust backend API
- Built as a **PWA** (Progressive Web App) so it can be "Added to Home
  Screen" on iOS and behaves like a native app icon, no App Store needed
- Plain CSS (or a small utility stylesheet) for the grid layout / book-shaped
  cards — kept lightweight so the app loads fast, no heavy bundler or
  build step required
- Served as static files directly from the Axum backend

**Hosting (mini PC)**
- Reverse proxy: **Caddy** (easiest, auto HTTPS) or Nginx
- HTTPS is required for a proper installable PWA on iOS
- Remote access without port-forwarding: **Tailscale** (private network
  between mini PC and both iPhones)
- Manga cover images stored locally on the mini PC, served as static files

## 5. Open Questions / To Decide
- How to add new manga/volumes to the collection: manual entry UI vs. import
  script vs. simple admin form
- Where cover images come from (manual upload vs. fetched from an API like
  MyAnimeList/AniList/Jikan)
- Whether "missing volumes" is manual toggle or computed from a known total
  volume count per series
- Backup strategy for the SQLite database on the mini PC

## 6. Rough Roadmap
1. Backend: CRUD API for manga + volumes, SQLite storage
2. Frontend: static home grid with hardcoded data, styled correctly
3. Wire frontend to backend API
4. Add PWA manifest + icons, test "Add to Home Screen" on both iPhones
5. Set up Caddy + Tailscale on the mini PC for always-on access
6. Polish: search/filter, sorting, missing-volume highlighting