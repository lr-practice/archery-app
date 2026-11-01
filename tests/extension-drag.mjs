// Prüft, dass die Extension Bedienelemente der Seite nicht zum Ziehen kapert (3.29.5):
// Lautstärkeregler über einem Video, kleine Knöpfe, Links über einem Bild.
// Seit 3.31.3 auch: Karten aus Kacheln, Text auf Hintergrundbildern und das
// eigene Ziehen der Seite (Kanban) bleiben bei der Seite, ohne Zonen.
//   node tests/extension-drag.mjs
import { chromium } from "playwright";
import http from "node:http";
import path from "node:path";
import os from "node:os";
import fs from "node:fs";
import assert from "node:assert/strict";

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const ext = path.join(root, "extension");
const png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

const html = `<!doctype html><meta charset=utf-8><title>Test</title><body style="margin:0">
<div style="position:relative;width:800px;height:450px;background:url(data:image/png;base64,${png}) center/cover">
  <a id=lnk href="#geklickt" style="position:absolute;inset:0"><span id=lspan style="position:absolute;inset:0"></span></a>
</div>
<div style="position:relative;width:640px;height:360px">
  <video poster="data:image/png;base64,${png}" style="width:640px;height:360px"></video>
  <div id=track role=slider style="position:absolute;right:20px;top:60px;width:40px;height:180px">
    <div id=knob style="position:absolute;left:4px;top:80px;width:32px;height:32px"></div>
  </div>
  <div id=small style="position:absolute;left:20px;top:20px;width:30px;height:30px"></div>
</div>
<script>window.hits = { track: 0, lnk: 0 };
track.addEventListener("pointerdown", () => hits.track++);
lnk.addEventListener("pointerdown", () => hits.lnk++);</script></body>`;
// Karte aus Bild-Kacheln unter einer leeren Fläche (Bauart Google Maps, OpenStreetMap),
// Text auf einem Hintergrundbild, eine Kanban-Karte und ein normales Bild
const tiles = Array.from({ length: 8 }, (_, i) => `<img draggable=false src="data:image/png;base64,${png}" style="position:absolute;width:200px;height:200px;left:${(i % 4) * 200}px;top:${Math.floor(i / 4) * 200}px">`).join("");
const html2 = `<!doctype html><meta charset=utf-8><title>Test 2</title><body style="margin:0">
<div style="position:relative;width:800px;height:400px">${tiles}<div id=pan style="position:absolute;inset:0"></div></div>
<div style="background:url(data:image/png;base64,${png}) center/cover;padding:20px"><div id=txt style="height:100px;font-size:20px">Text auf einem Hintergrundbild, der markierbar bleiben soll.</div></div>
<div id=card draggable=true style="width:200px;height:60px;background:#eee">Kanban-Karte</div>
<img id=pic src="data:image/png;base64,${png}" style="width:200px;height:120px">
<script>window.hits = { pan: 0 };
pan.addEventListener("pointerdown", () => hits.pan++);
card.addEventListener("dragstart", (e) => e.dataTransfer.setData("text/plain", "karte"));</script></body>`;
const server = http.createServer((req, res) => res.end(req.url === "/2" ? html2 : html)).listen(0);
const url = `http://127.0.0.1:${server.address().port}/`;

const profile = fs.mkdtempSync(path.join(os.tmpdir(), "archiver-ext-"));
const ctx = await chromium.launchPersistentContext(profile, {
  channel: "chromium",
  headless: true,
  args: [`--disable-extensions-except=${ext}`, `--load-extension=${ext}`],
});
let failed = 0;
async function check(name, fn) {
  try {
    await fn();
    console.log(`ok   ${name}`);
  } catch (e) {
    failed++;
    console.log(`FAIL ${name}\n     ${e.message}`);
  }
}

async function press(page, sel) {
  const b = await page.locator(sel).boundingBox();
  await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2);
  await page.mouse.down();
  const draggable = await page.evaluate((s) => document.querySelector(s).getAttribute("draggable"), sel);
  await page.mouse.up();
  return draggable;
}

try {
  if (!ctx.serviceWorkers().length) await ctx.waitForEvent("serviceworker", { timeout: 10000 });
  const page = await ctx.newPage();
  await page.goto(url);
  await page.waitForTimeout(800);

  await check("Lautstärkeregler über dem Video bleibt ein Regler", async () => {
    assert.equal(await press(page, "#knob"), null);
    assert.equal(await page.evaluate(() => hits.track), 1);
  });

  await check("kleiner Knopf über dem Video wird nicht ziehbar", async () => {
    assert.equal(await press(page, "#small"), null);
  });

  await check("Link über einem Bild bekommt Druck und Klick", async () => {
    await page.click("#lspan");
    assert.equal(await page.evaluate(() => hits.lnk), 1);
    assert.equal(await page.evaluate(() => location.hash), "#geklickt");
  });

  const page2 = await ctx.newPage();
  await page2.goto(url + "2");
  await page2.waitForTimeout(800);

  await check("Karte aus Kacheln: Verschieben bleibt bei der Seite", async () => {
    assert.equal(await press(page2, "#pan"), null);
    assert.equal(await page2.evaluate(() => hits.pan), 1);
  });

  await check("Text auf einem Hintergrundbild bleibt markierbar", async () => {
    assert.equal(await press(page2, "#txt"), null);
  });

  async function dragFrom(sel) {
    const b = await page2.locator(sel).boundingBox();
    await page2.mouse.move(b.x + b.width / 2, b.y + b.height / 2);
    await page2.mouse.down();
    for (let i = 1; i <= 6; i++) await page2.mouse.move(b.x + b.width / 2 + i * 30, b.y + b.height / 2 - i * 20);
    await page2.waitForTimeout(500);
    const zones = await page2.evaluate(() => !!document.querySelector("archiver-overlay"));
    await page2.mouse.up();
    return zones;
  }

  await check("Ziehen der Seite (Kanban-Karte) blendet keine Zonen ein", async () => {
    assert.equal(await dragFrom("#card"), false);
  });

  await check("Bild ziehen blendet die Zonen ein", async () => {
    assert.equal(await dragFrom("#pic"), true);
  });
} finally {
  await ctx.close();
  server.close();
}
if (failed) process.exit(1);
