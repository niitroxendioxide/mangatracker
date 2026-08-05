const API_BASE = ""; // http://localhost:3000

  const gridEl = document.getElementById("manga-grid");
  const statusEl = document.getElementById("status");
  const formEl = document.getElementById("add-form");

  const backdropEl = document.getElementById("modal-backdrop");
  const modalTitleEl = document.getElementById("modal-title");
  const modalProgressEl = document.getElementById("modal-progress");
  const modalVolumesEl = document.getElementById("modal-volumes");
  const modalCloseEl = document.getElementById("modal-close");

  let currentManga = null; // the manga object currently open in the modal

  function setStatus(msg, isError = false) {
    statusEl.textContent = msg;
    statusEl.style.color = isError ? "var(--stamp)" : "var(--gray)";
  }

  async function fetchAllManga() {
    const res = await fetch(`${API_BASE}/manga`);
    if (!res.ok) throw new Error(`Failed to load (${res.status})`);
    return res.json();
  }

  async function createManga(name, volumeCount, price) {
    const res = await fetch(`${API_BASE}/manga/create`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name, volume_count: volumeCount, last_price: price }),
    });
    if (!res.ok) throw new Error(`Failed to create (${res.status})`);
    return res.json();
  }

  async function setVolumeOwned(mangaId, volume, state) {
    const res = await fetch(`${API_BASE}/manga/update/${mangaId}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ volume, state }),
    });
    if (!res.ok) throw new Error(`Failed to update (${res.status})`);
  }

  // --- Grid tile ---

  function renderTile(manga) {
    const tile = document.createElement("button");
    tile.className = "manga-tile";
    tile.type = "button";

    const cover = document.createElement("div");
    cover.className = "manga-cover";
    if (manga.cover_image_path) {
      const img = document.createElement("img");
      img.referrerPolicy="no-referrer"
      img.src = `${manga.cover_image_path}`;
      img.alt = "";
      cover.appendChild(img);
    } else {
      cover.textContent = manga.name.charAt(0).toUpperCase();
    }

    console.log(manga);

    const info = document.createElement("div");
    info.className = "manga-info";
    info.innerHTML = `
      <p class="manga-title">${manga.name}</p>
      <p class="manga-owned-count">Volumenes Comprados: ${manga.owned_volumes.length}</p>
      <p class="${manga.last_price === 0 ? "unknown-price" : "price"}">
           ${manga.last_price === 0 ? "Desconocido" : `AR$ ${manga.last_price}`}
      </p>
    `;

    tile.appendChild(cover);
    tile.appendChild(info);
    tile.addEventListener("click", () => openModal(manga));
    return tile;
  }

  // --- Modal ---

  function openModal(manga) {
    currentManga = manga;
    modalTitleEl.textContent = manga.name;
    updateModalProgress();

    modalVolumesEl.innerHTML = "";
    for (let v = 1; v <= manga.volume_count; v++) {
      const owned = manga.owned_volumes.includes(v);
      const vEl = document.createElement("div");
      vEl.className = "volume" + (owned ? " owned" : "");
      vEl.textContent = v;
      vEl.addEventListener("click", () => onToggleVolume(v, vEl));
      modalVolumesEl.appendChild(vEl);
    }

    backdropEl.classList.add("open");
  }

  function closeModal() {
    backdropEl.classList.remove("open");
    currentManga = null;
  }

  function updateModalProgress() {
    modalProgressEl.textContent =
      `${currentManga.owned_volumes.length} / ${currentManga.volume_count} owned`;
  }

  async function onToggleVolume(volumeNumber, vEl) {
    const manga = currentManga;
    const newState = !vEl.classList.contains("owned");
    vEl.classList.toggle("owned", newState); // optimistic UI update

    try {
      await setVolumeOwned(manga.id, volumeNumber, newState);

      if (newState) {
        manga.owned_volumes.push(volumeNumber);
      } else {
        manga.owned_volumes = manga.owned_volumes.filter(v => v !== volumeNumber);
      }
      updateModalProgress();

      // keep the grid tile's count in sync for when the modal closes
      const tileCountEl = [...gridEl.children]
        .find(t => t.dataset.mangaId == manga.id)
        ?.querySelector(".manga-owned-count");
      if (tileCountEl) tileCountEl.textContent = `Volumes owned: ${manga.owned_volumes.length}`;
    } catch (err) {
      vEl.classList.toggle("owned", !newState); // revert on failure
      setStatus(err.message, true);
    }
  }

  modalCloseEl.addEventListener("click", closeModal);
  backdropEl.addEventListener("click", (e) => {
    if (e.target === backdropEl) closeModal();
  });

  // --- Load + add ---

  async function loadAndRender() {
    setStatus("Loading…");
    try {
      const mangas = await fetchAllManga();
      gridEl.innerHTML = "";
      mangas.forEach(m => {
        const tile = renderTile(m);
        tile.dataset.mangaId = m.id;
        gridEl.appendChild(tile);
      });
      setStatus("");
    } catch (err) {
      setStatus(err.message, true);
    }
  }

  formEl.addEventListener("submit", async (e) => {
    e.preventDefault();
    const name = formEl.name.value.trim();
    const volumeCount = parseInt(formEl.volume_count.value, 10);
    const price = parseInt(formEl.last_price.value, 10);

    if (!name || !volumeCount || !price) return;
    
    console.log("Sending price: ", price);
    setStatus("Adding…");
    try {
      await createManga(name, volumeCount, price);
      formEl.reset();
      await loadAndRender();
    } catch (err) {
      setStatus(err.message, true);
    }
  });

  loadAndRender();