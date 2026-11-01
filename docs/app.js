import init, { algorithms, packRects } from "./pkg/open_texture_packer_wasm.js";

const ALL_SET = { id: "all", label: "All sprites" };
const DEFAULTS = { set: "dj_fox", sizes: "mixed", max: "4096" };
const MIN_MIXED_SCALE = 0.25;
const OUTLINE_COLOR = "#ff2d78";
const MIN_LABEL_WIDTH = 22;

const elements = {
  controls: document.getElementById("controls"),
  set: document.getElementById("set"),
  sizes: document.getElementById("sizes"),
  max: document.getElementById("max"),
  outlines: document.getElementById("outlines"),
  status: document.getElementById("status"),
  summary: document.getElementById("summary"),
  cards: document.getElementById("cards"),
  tooltip: document.getElementById("tooltip"),
};

const imageCache = new Map();
let sets = [];
let algorithmList = [];
let current = null;
let renderToken = 0;

function el(tag, props = {}, ...children) {
  const node = Object.assign(document.createElement(tag), props);
  node.append(...children);
  return node;
}

function setStatus(message, isError = false) {
  elements.status.textContent = message;
  elements.status.classList.toggle("error", isError);
}

/** The same sprite always gets the same scale, so results can be compared. */
function mixedScale(index) {
  let x = Math.imul(index + 1, 2654435761) >>> 0;
  x ^= x >>> 13;
  x = Math.imul(x, 1597334677) >>> 0;
  x ^= x >>> 16;
  return MIN_MIXED_SCALE + ((x >>> 0) / 0xffffffff) * (1 - MIN_MIXED_SCALE);
}

function buildSprites(setId, sizeMode) {
  const source = sets.find((set) => set.id === setId) ?? sets[0];
  return source.sprites.map((sprite, index) => {
    const scale = sizeMode === "mixed" ? mixedScale(index) : 1;
    return {
      ...sprite,
      width: Math.max(1, Math.round(sprite.width * scale)),
      height: Math.max(1, Math.round(sprite.height * scale)),
    };
  });
}

function loadImage(path) {
  if (!imageCache.has(path)) {
    imageCache.set(path, new Promise((resolve, reject) => {
      const image = new Image();
      image.onload = () => resolve(image);
      image.onerror = () => {
        imageCache.delete(path);
        reject(new Error(`Could not load ${path}`));
      };
      image.src = path;
    }));
  }
  return imageCache.get(path);
}

function packAll(sprites, max) {
  const sizes = sprites.map(({ width, height }) => ({ width, height }));
  return algorithmList.map(({ id, name }) => {
    const started = performance.now();
    try {
      const result = packRects(sizes, id, max, max);
      return { ...result, time: performance.now() - started };
    } catch (error) {
      return { algorithm: id, name, error: error.message };
    }
  });
}

function bestValues(results) {
  const valid = results.filter((result) => !result.error);
  return {
    sheets: Math.min(...valid.map((result) => result.sheets.length)),
    area: Math.min(...valid.map((result) => result.sheetArea)),
  };
}

function isSameLayoutCost(results) {
  const valid = results.filter((result) => !result.error);
  return valid.length > 1 && valid.every((result) =>
    result.sheets.length === valid[0].sheets.length && result.sheetArea === valid[0].sheetArea);
}

function formatPercent(value) {
  return `${(value * 100).toFixed(1)}%`;
}

function formatTime(ms) {
  return ms < 1 ? "< 1 ms" : `${ms.toFixed(1)} ms`;
}

function sheetSizes(result) {
  return result.sheets.map((sheet) => `${sheet.width}×${sheet.height}`).join(", ");
}

function renderSummary(results) {
  const best = bestValues(results);
  const rows = results.map((result) => {
    if (result.error) {
      return el("tr", {},
        el("td", { textContent: result.name }),
        el("td", { className: "failed", colSpan: 5, textContent: result.error }));
    }
    const bestSheets = result.sheets.length === best.sheets;
    const bestArea = bestSheets && result.sheetArea === best.area;
    return el("tr", {},
      el("td", { textContent: result.name }),
      el("td", { className: `number${bestSheets ? " best" : ""}`, textContent: result.sheets.length }),
      el("td", { textContent: sheetSizes(result) }),
      el("td", { className: `number${bestArea ? " best" : ""}`, textContent: result.sheetArea.toLocaleString("en-US") }),
      el("td", { className: `number${bestArea ? " best" : ""}`, textContent: formatPercent(result.occupancy) }),
      el("td", { className: "number", textContent: formatTime(result.time) }));
  });
  elements.summary.tBodies[0].replaceChildren(...rows);
  elements.summary.hidden = false;
}

function createCard(result) {
  if (result.error) {
    return el("article", { className: "card" },
      el("h2", { textContent: result.name }),
      el("p", { className: "stats", textContent: result.error }));
  }

  const canvases = result.sheets.map((sheet, index) => {
    const canvas = el("canvas");
    canvas.packedSheet = sheet;
    const caption = el("figcaption", { textContent: `Sheet ${index + 1}: ${sheet.width} × ${sheet.height}` });
    canvas.setAttribute("role", "img");
    canvas.setAttribute("aria-label", `${result.name}, sheet ${index + 1}: ${sheet.placements.length} sprites on ${sheet.width} by ${sheet.height} pixels`);
    return el("figure", {}, canvas, caption);
  });

  const stats = `${result.sheets.length} ${result.sheets.length === 1 ? "sheet" : "sheets"} · ${formatPercent(result.occupancy)} occupancy`;
  return el("article", { className: "card" },
    el("h2", { textContent: result.name }),
    el("p", { className: "stats", textContent: stats }),
    el("div", { className: "sheets" }, ...canvases));
}

/** One scale for all cards, so that sheet sizes can be compared by eye. */
function displayScale(results) {
  const card = elements.cards.querySelector(".sheets");
  if (!card) return 1;
  const largest = Math.max(...results.flatMap((result) =>
    (result.sheets ?? []).map((sheet) => Math.max(sheet.width, sheet.height))));
  return Math.min(1, card.clientWidth / largest);
}

function drawSheet(canvas, images, scale) {
  const sheet = canvas.packedSheet;
  const ratio = window.devicePixelRatio || 1;
  canvas.width = Math.max(1, Math.round(sheet.width * scale * ratio));
  canvas.height = Math.max(1, Math.round(sheet.height * scale * ratio));
  canvas.style.width = `${sheet.width * scale}px`;
  canvas.style.height = `${sheet.height * scale}px`;
  canvas.displayScale = scale;

  const context = canvas.getContext("2d");
  context.setTransform(scale * ratio, 0, 0, scale * ratio, 0, 0);
  context.imageSmoothingQuality = "high";

  for (const placement of sheet.placements) {
    const image = images[placement.index];
    if (image) {
      context.drawImage(image, placement.x, placement.y, placement.width, placement.height);
    } else {
      context.fillStyle = "rgba(128, 128, 128, 0.35)";
      context.fillRect(placement.x, placement.y, placement.width, placement.height);
    }
  }

  if (!elements.outlines.checked) return;

  context.lineWidth = 1 / (scale * ratio);
  context.strokeStyle = OUTLINE_COLOR;
  context.font = `600 ${11 / scale}px system-ui, sans-serif`;
  context.textBaseline = "top";
  for (const placement of sheet.placements) {
    context.strokeRect(placement.x, placement.y, placement.width, placement.height);
    if (placement.width * scale >= MIN_LABEL_WIDTH) {
      const x = placement.x + 3 / scale;
      const y = placement.y + 2 / scale;
      const label = String(placement.index + 1);
      context.lineWidth = 3 / scale;
      context.strokeStyle = "#000000";
      context.strokeText(label, x, y);
      context.fillStyle = "#ffffff";
      context.fillText(label, x, y);
      context.lineWidth = 1 / (scale * ratio);
      context.strokeStyle = OUTLINE_COLOR;
    }
  }
}

function drawAll() {
  if (!current) return;
  const scale = displayScale(current.results);
  for (const canvas of elements.cards.querySelectorAll("canvas")) {
    drawSheet(canvas, current.images, scale);
  }
}

function placementAt(canvas, event) {
  const rect = canvas.getBoundingClientRect();
  if (!rect.width) return null;
  const sheet = canvas.packedSheet;
  const x = ((event.clientX - rect.left) / rect.width) * sheet.width;
  const y = ((event.clientY - rect.top) / rect.height) * sheet.height;
  return sheet.placements.find((p) =>
    x >= p.x && x < p.x + p.width && y >= p.y && y < p.y + p.height);
}

function hideTooltip() {
  elements.tooltip.hidden = true;
}

function showTooltip(event) {
  const canvas = event.target;
  const placement = canvas instanceof HTMLCanvasElement ? placementAt(canvas, event) : null;
  if (!placement || !current) {
    hideTooltip();
    return;
  }

  const { tooltip } = elements;
  const sprite = current.sprites[placement.index];
  tooltip.textContent = `#${placement.index + 1} ${sprite.name}\nx ${placement.x}, y ${placement.y}\n${placement.width} × ${placement.height}`;
  tooltip.hidden = false;

  const margin = 12;
  const left = Math.min(event.clientX + margin, window.innerWidth - tooltip.offsetWidth - margin);
  const top = Math.min(event.clientY + margin, window.innerHeight - tooltip.offsetHeight - margin);
  tooltip.style.left = `${Math.max(margin, left)}px`;
  tooltip.style.top = `${Math.max(margin, top)}px`;
}

function readHash() {
  const params = new URLSearchParams(location.hash.slice(1));
  const pick = (select, key) => {
    const value = params.get(key);
    const allowed = [...select.options].some((option) => option.value === value);
    select.value = allowed ? value : DEFAULTS[key];
  };
  pick(elements.set, "set");
  pick(elements.sizes, "sizes");
  pick(elements.max, "max");
}

function writeHash() {
  const params = new URLSearchParams({
    set: elements.set.value,
    sizes: elements.sizes.value,
    max: elements.max.value,
  });
  history.replaceState(null, "", `#${params}`);
}

async function render() {
  const token = ++renderToken;
  hideTooltip();

  const sprites = buildSprites(elements.set.value, elements.sizes.value);
  const results = packAll(sprites, Number(elements.max.value));
  const images = new Array(sprites.length).fill(null);
  current = { sprites, results, images };

  renderSummary(results);
  elements.cards.replaceChildren(...results.map(createCard));
  drawAll();

  let loaded = 0;
  const showProgress = () => setStatus(`Loading images: ${loaded} of ${sprites.length}`);
  showProgress();

  const outcomes = await Promise.allSettled(sprites.map(async (sprite, index) => {
    images[index] = await loadImage(sprite.path);
    loaded += 1;
    if (token === renderToken) showProgress();
  }));
  if (token !== renderToken) return;

  drawAll();

  const failed = outcomes.filter((outcome) => outcome.status === "rejected").length;
  if (failed > 0) {
    setStatus(`${failed} of ${sprites.length} images could not be loaded. They are shown as gray boxes.`, true);
  } else if (results.every((result) => result.error)) {
    setStatus("No layout is possible with this sheet size.", true);
  } else {
    const hint = elements.sizes.value === "original" ? " Choose Mixed sizes to see differences." : "";
    setStatus(isSameLayoutCost(results)
      ? `${sprites.length} sprites. All algorithms need the same area for these sprites.${hint}`
      : `${sprites.length} sprites. Green marks the fewest sheets and the smallest area.`);
  }
}

async function start() {
  setStatus("Loading…");
  await init();
  algorithmList = algorithms();

  const response = await fetch("assets/manifest.json");
  if (!response.ok) throw new Error(`Could not load the sprite list (HTTP ${response.status})`);
  const manifest = await response.json();

  sets = [...manifest.sets, { ...ALL_SET, sprites: manifest.sets.flatMap((set) => set.sprites) }];
  elements.set.append(...sets.map((set) =>
    el("option", { value: set.id, textContent: `${set.label} (${set.sprites.length})` })));

  readHash();

  elements.controls.addEventListener("submit", (event) => event.preventDefault());
  for (const select of [elements.set, elements.sizes, elements.max]) {
    select.addEventListener("change", () => {
      writeHash();
      render();
    });
  }
  elements.outlines.addEventListener("change", drawAll);
  elements.cards.addEventListener("pointermove", showTooltip);
  elements.cards.addEventListener("pointerdown", showTooltip);
  elements.cards.addEventListener("pointerleave", hideTooltip);

  let resizeFrame = 0;
  window.addEventListener("resize", () => {
    cancelAnimationFrame(resizeFrame);
    resizeFrame = requestAnimationFrame(drawAll);
  });
  window.addEventListener("hashchange", () => {
    readHash();
    render();
  });

  await render();
}

start().catch((error) => {
  setStatus(`The demo could not start: ${error.message}`, true);
});
