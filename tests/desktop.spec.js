import { expect, test } from "@playwright/test";

// The real backend is covered by `cargo test`; here a stub of window.__TAURI__ exercises the desktop UI.
const stub = () => {
  const st = { entries: [], total_sats: 0, scanning: false, watching: false };
  const KIND = { 1: "P2pkh", 3: "P2sh", b: "Bech32" };
  const handlers = {
    get_state: () => structuredClone(st),
    add_text: ({ text }) => {
      const toks = text.split(/[\s,;]+/).filter(Boolean);
      const rep = { added: 0, duplicates: 0, invalid: [] };
      for (const t of toks) {
        if (!/^(1|3|bc1)[A-Za-z0-9]{20,}$/.test(t)) rep.invalid.push(t);
        else if (st.entries.some((e) => e.address === t)) rep.duplicates++;
        else { st.entries.push({ address: t, kind: KIND[t[0]], sats: null, error: null, checked: null }); rep.added++; }
      }
      return rep;
    },
    remove: ({ address }) => { st.entries = st.entries.filter((e) => e.address !== address); },
    clear: () => { st.entries = []; },
    check_now: () => { st.entries.forEach((e, i) => Object.assign(e, { sats: i === 0 ? 123456789 : 0, checked: 1700000000 })); st.total_sats = 123456789; return { checked: st.entries.length, errors: 0, alerts: 1, problems: [] }; },
    set_watch: ({ on }) => { st.watching = on; },
    get_settings: () => ({ telegram_chat_id: "42", smtp_host: "smtp.gmail.com", smtp_port: 465, interval_secs: 60, telegram_token_set: true, email_password_set: false }),
    save_settings: () => {},
    export_list: ({ format }) => (window.__cancelExport ? null : `/tmp/list.${format}`),
    test_alert: () => { if (window.__alertFails) throw "smtp: refused"; },
  };
  window.__TAURI__ = { core: { invoke: async (c, a) => handlers[c](a) }, event: { listen: async (n, f) => { (window.__ev ??= {})[n] = f; return () => {}; } } };
};

test.beforeEach(async ({ page }) => {
  await page.addInitScript(stub);
  await page.goto("http://127.0.0.1:47311/");
});

test("add, check, total, remove, clear", async ({ page }) => {
  await page.locator("#many").fill("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa 3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy junk");
  await page.getByRole("button", { name: "Add all" }).click();
  await expect(page.locator("#report")).toContainText("2 added");
  await expect(page.locator("#report")).toContainText("1 invalid: junk");
  await page.getByRole("button", { name: "Check now" }).click();
  await expect(page.locator("#total-btc")).toHaveText("1.23456789");
  await expect(page.locator("#rows tr").first()).toHaveClass(/funded/);
  await page.getByRole("button", { name: /^Remove 3J98/ }).click();
  await expect(page.locator("#count")).toHaveText("1");
  page.once("dialog", (d) => d.accept());
  await page.getByRole("button", { name: "Clear all" }).click();
  await expect(page.locator("#empty")).toBeVisible();
});

test("watch toggle exposes state and settings dialog never echoes secrets", async ({ page }) => {
  await page.getByRole("button", { name: "Watch: off" }).click();
  await expect(page.getByRole("button", { name: "Watch: on" })).toHaveAttribute("aria-pressed", "true");
  await page.getByRole("button", { name: "Settings" }).click();
  await expect(page.locator("[name=telegram_token]")).toHaveValue("");
  await expect(page.locator("#tok-set")).toHaveText("(stored)");
  await expect(page.locator("[name=telegram_chat_id]")).toHaveValue("42");
});

test("watch-scan problems surface in the report", async ({ page }) => {
  await page.evaluate(() => window.__ev["scan-problems"]({ payload: ["alert failed: smtp: refused"] }));
  await expect(page.locator("#report")).toContainText("alert failed: smtp: refused");
  await expect(page.locator("#report")).toHaveClass(/bad/);
});

test("export reports the path, and stays quiet when the dialog is cancelled", async ({ page }) => {
  await page.locator("#many").fill("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
  await page.getByRole("button", { name: "Add all" }).click();
  await page.locator("#export-csv").click();
  await expect(page.locator("#report")).toContainText("Saved to /tmp/list.csv");
  await page.evaluate(() => { window.__cancelExport = true; document.getElementById("report").textContent = ""; });
  await page.locator("#export-txt").click();
  await expect(page.locator("#report")).toHaveText("");
});

test("test alert shows success or the failure reason", async ({ page }) => {
  await page.getByRole("button", { name: "Settings" }).click();
  await page.getByRole("button", { name: "Send test alert" }).click();
  await expect(page.locator("#set-msg")).toHaveText("Test alert sent.");
  await page.evaluate(() => (window.__alertFails = true));
  await page.getByRole("button", { name: "Send test alert" }).click();
  await expect(page.locator("#set-msg")).toContainText("smtp: refused");
});

for (const width of [320, 1024]) {
  test(`no horizontal scroll at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 800 });
    await page.locator("#many").fill("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq");
    await page.getByRole("button", { name: "Add all" }).click();
    await page.getByRole("button", { name: "Check now" }).click();
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(width);
    await page.screenshot({ path: `shots/desktop-${width}.png`, fullPage: true });
  });
}
