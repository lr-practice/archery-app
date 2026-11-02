// Prüft, dass eine Webseite die Extension nicht fernsteuern kann (0.31.0, Befunde E1/E2).
// Lädt extension/ entpackt in Chromium und vergleicht echte mit erzeugten Events.
//   node tests/extension-security.mjs
import { chromium } from "playwright";
import http from "node:http";
import path from "node:path";
import os from "node:os";
import fs from "node:fs";
import assert from "node:assert/strict";

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const ext = path.join(root, "extension");

const html = `<!doctype html><meta charset=utf-8><title>Test</title>
<body style="height:2000px"><img id=pic src="data:image/gif;base64,R0lGODlhAQABAAAAACw=" width=200 height=200></body>`;
const server = http.createServer((_, res) => res.end(html)).listen(0);
const url = `http://127.0.0.1:${server.address().port}/`;

const profile = fs.mkdtempSync(path.join(os.tmpdir(), "archiver-ext-"));
const ctx = await chromium.launchPersistentContext(profile, {
  channel: "chromium", // volles Chromium; die Headless-Shell lädt keine Extensions
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

try {
  if (!ctx.serviceWorkers().length) await ctx.waitForEvent("serviceworker", { timeout: 10000 });
  const page = await ctx.newPage();
  await page.goto(url);
  await page.waitForTimeout(800); // Content-Scripts laufen nach document_idle

  const hasOverlay = () => page.evaluate(() => !!document.querySelector("archery-overlay"));

  await check("erzeugtes Tastenkürzel öffnet kein Overlay", async () => {
    await page.evaluate(() =>
      document.dispatchEvent(
        new KeyboardEvent("keydown", { key: "a", code: "KeyA", altKey: true, shiftKey: true, bubbles: true })
      )
    );
    await page.waitForTimeout(500);
    assert.equal(await hasOverlay(), false);
  });

  await check("erzeugtes Ziehen blendet keine Zonenleiste ein und sichert nichts", async () => {
    const before = await ctx.serviceWorkers()[0].evaluate(() => chrome.downloads.search({}).then((d) => d.length));
    await page.evaluate(() => {
      const dt = new DataTransfer();
      dt.setData("text/uri-list", "https://example.com/x.jpg");
      const opts = { bubbles: true, cancelable: true, dataTransfer: dt, clientX: 100, clientY: innerHeight - 5 };
      document.dispatchEvent(new DragEvent("dragenter", opts));
      document.dispatchEvent(new DragEvent("dragover", opts));
      document.dispatchEvent(new DragEvent("drop", opts));
    });
    await page.waitForTimeout(800);
    assert.equal(await hasOverlay(), false, "Overlay/Leiste wurde angelegt");
    const after = await ctx.serviceWorkers()[0].evaluate(() => chrome.downloads.search({}).then((d) => d.length));
    assert.equal(after, before, "Download wurde ausgelöst");
  });

  await check("echtes Tastenkürzel öffnet das Overlay, die Seite sieht nicht hinein", async () => {
    await page.bringToFront();
    await page.mouse.click(5, 5);
    await page.keyboard.press("Alt+Shift+KeyA");
    await page.waitForFunction(() => !!document.querySelector("archery-overlay"), null, { timeout: 5000 });
    const open = await page.evaluate(() => document.querySelector("archery-overlay").shadowRoot);
    assert.equal(open, null, "shadowRoot ist von der Seite aus lesbar");
  });
} finally {
  await ctx.close();
  server.close();
  fs.rmSync(profile, { recursive: true, force: true });
}
if (failed) {
  console.log(`${failed} fehlgeschlagen`);
  process.exit(1);
}
