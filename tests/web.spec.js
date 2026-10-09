import { expect, test } from "@playwright/test";
import { readFileSync } from "node:fs";

const GENESIS = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
const SAMPLES = readFileSync(new URL("./sample-addresses.txt", import.meta.url), "utf8").trim().split("\n");
const stats = (chain, pending = 0) => ({ chain_stats: { funded_txo_sum: chain, spent_txo_sum: 0 }, mempool_stats: { funded_txo_sum: pending, spent_txo_sum: 0 } });

test.describe("web app", () => {
  test.beforeEach(async ({ page }) => {
    await page.route("https://mempool.space/api/address/*", (r) => {
      const a = r.request().url().split("/").pop();
      return a === GENESIS ? r.fulfill({ json: stats(5_000_000_000, 10) }) : r.fulfill({ status: 429, body: "slow down" });
    });
    await page.goto("/app/");
  });

  test("checks a pasted list, shows total, flags invalid and errors", async ({ page }) => {
    await page.locator("#many").fill(`${GENESIS}\n${GENESIS}\nbogus,1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNb\n${SAMPLES[0]}`);
    await page.getByRole("button", { name: "Add all" }).click();
    await expect(page.locator("#report")).toContainText("2 added");
    await expect(page.locator("#report")).toContainText("1 duplicate");
    await expect(page.locator("#report")).toContainText("2 invalid");
    await page.getByRole("button", { name: "Check balances" }).click();
    await expect(page.locator("#total-btc")).toHaveText("50.00000010", { timeout: 15000 });
    await expect(page.locator("#rows tr").first()).toContainText("Funded");
    await expect(page.locator("#rows tr").nth(1)).toContainText("rate limited");
  });

  test("accepts a dropped-style .txt upload and rejects other types", async ({ page }) => {
    await page.locator("#file").setInputFiles({ name: "address.txt", mimeType: "text/plain", buffer: Buffer.from(SAMPLES.join("\n")) });
    await expect(page.locator("#count")).toHaveText(String(SAMPLES.length));
    await page.locator("#file").setInputFiles({ name: "x.png", mimeType: "image/png", buffer: Buffer.from("x") });
    await expect(page.locator("#report")).toContainText("only .txt and .csv");
  });

  test("drop zone is keyboard reachable and drag-drop adds addresses", async ({ page }) => {
    await page.locator("#drop").focus();
    await expect(page.locator("#drop")).toBeFocused();
    const dt = await page.evaluateHandle((a) => { const d = new DataTransfer(); d.items.add(new File([a], "a.csv", { type: "text/csv" })); return d; }, GENESIS);
    await page.dispatchEvent("body", "drop", { dataTransfer: dt });
    await expect(page.locator("#count")).toHaveText("1");
  });

  test("no horizontal scroll at 320px", async ({ page }) => {
    await page.setViewportSize({ width: 320, height: 700 });
    await page.locator("#many").fill(SAMPLES.join("\n"));
    await page.getByRole("button", { name: "Add all" }).click();
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(320);
  });
});

test.describe("web app failure handling", () => {
  const check = async (page, body) => {
    await page.route("https://mempool.space/api/address/*", body);
    await page.goto("/app/");
    await page.locator("#one").fill(GENESIS);
    await page.getByRole("button", { name: "Add address" }).click();
    await page.getByRole("button", { name: "Check balances" }).click();
  };

  test("malformed numbers show an error, never NaN, and controls re-enable", async ({ page }) => {
    await check(page, (r) => r.fulfill({ json: { chain_stats: { funded_txo_sum: "x", spent_txo_sum: 0 }, mempool_stats: {} } }));
    await expect(page.locator("#rows tr").first()).toContainText("unexpected response");
    await expect(page.locator("body")).not.toContainText("NaN");
    await expect(page.getByRole("button", { name: "Check balances" })).toBeEnabled();
    await expect(page.locator("#clear")).toBeEnabled();
  });

  test("a stalled request times out instead of hanging", async ({ page }) => {
    await check(page, () => {}); // never answers
    await expect(page.locator("#rows tr").first()).toContainText("timed out", { timeout: 20000 });
    await expect(page.getByRole("button", { name: "Check balances" })).toBeEnabled();
  });
});

test.describe("web app (live network)", () => {
  test.skip(!process.env.LIVE, "set LIVE=1 to hit mempool.space");
  test("looks up a real address", async ({ page }) => {
    await page.goto("/app/");
    await page.locator("#one").fill(GENESIS);
    await page.getByRole("button", { name: "Add address" }).click();
    await page.getByRole("button", { name: "Check balances" }).click();
    await expect(page.locator("#rows tr").first()).toContainText(/Funded|Empty/, { timeout: 20000 });
    await expect(page.locator("#rows tr").first()).not.toContainText("Error");
    await page.screenshot({ path: "shots/web-live.png", fullPage: true });
  });
});

test.describe("landing", () => {
  const release = { tag_name: "v2.0.0", assets: ["a.exe", "a.msi", "a.dmg", "a.AppImage", "a.deb", "a.rpm"].map((n) => ({ name: n, browser_download_url: `https://example.test/${n}` })) };

  test("download buttons come from the GitHub API", async ({ page }) => {
    await page.route("https://api.github.com/**", (r) => r.fulfill({ json: release }));
    await page.goto("/");
    await expect(page.locator("#dl-status")).toContainText("v2.0.0");
    await expect(page.getByRole("link", { name: ".dmg" })).toHaveAttribute("href", "https://example.test/a.dmg");
    await expect(page.getByRole("link", { name: ".AppImage" })).toHaveAttribute("href", "https://example.test/a.AppImage");
  });

  test("falls back to the releases page when the API fails", async ({ page }) => {
    await page.route("https://api.github.com/**", (r) => r.fulfill({ status: 403, body: "{}" }));
    await page.goto("/");
    await expect(page.locator("#dl-status")).toContainText("Could not reach");
    for (const l of await page.locator(".links a").all()) await expect(l).toHaveAttribute("href", "https://github.com/jonacarbi/btc-balance-checker/releases");
  });

  test.describe("without JavaScript", () => {
    test.use({ javaScriptEnabled: false });
    test("static buttons still open the releases page", async ({ page }) => {
      await page.goto("/");
      await expect(page.locator(".links a")).toHaveCount(6);
      for (const l of await page.locator(".links a").all()) await expect(l).toHaveAttribute("href", "https://github.com/jonacarbi/btc-balance-checker/releases");
    });
  });

  test.describe("macOS visitor", () => {
    test.use({ userAgent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 Safari/605.1.15" });
    test("highlights the build for this platform", async ({ page }) => {
      await page.route("https://api.github.com/**", (r) => r.fulfill({ json: release }));
      await page.goto("/");
      await expect(page.locator('[data-os="macos"]')).toHaveClass(/mine/);
      await expect(page.locator('[data-os="windows"]')).not.toHaveClass(/mine/);
    });
  });

  for (const width of [320, 375, 768, 1440]) {
    test(`no horizontal overflow and screenshot at ${width}px`, async ({ page }) => {
      await page.route("https://api.github.com/**", (r) => r.fulfill({ json: release }));
      await page.setViewportSize({ width, height: 900 });
      await page.goto("/");
      await expect(page.locator("#dl-status")).toContainText("v2.0.0");
      expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(width);
      await page.screenshot({ path: `shots/landing-${width}.png`, fullPage: true });
    });
  }

  test("agent twins exist", async ({ request }) => {
    for (const p of ["/llms.txt", "/index.md", "/app/index.md", "/favicon.ico", "/favicon.svg", "/logo.svg"]) expect((await request.get(p)).ok(), p).toBe(true);
  });
});
