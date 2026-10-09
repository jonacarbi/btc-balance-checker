import { parseBulk } from "./addr.js";

const $ = (id) => document.getElementById(id);
const API = "https://mempool.space/api/address/";
const MAX_ADDRESSES = 500;
const MAX_FILE = 5 * 1024 * 1024;
const GAP_MS = 350;
const btc = (s) => `${Math.floor(s / 1e8)}.${String(s % 1e8).padStart(8, "0")}`;
const fmt = (n) => n.toLocaleString("en-US");

/** @type {{address:string, kind:string, sats:number|null, error:string|null}[]} */
let rows = [];
let busy = false;

const cell = (text, cls, label) => {
  const td = document.createElement("td");
  td.textContent = text;
  if (cls) td.className = cls;
  if (label) td.dataset.label = label;
  return td;
};

function render() {
  const tr = (r, i) => {
    const el = document.createElement("tr");
    const [st, cls] = r.error ? ["Error: " + r.error, "status-err"] : r.sats === null ? ["Pending", "status-wait"] : r.sats > 0 ? ["Funded", "status-ok"] : ["Empty", "status-wait"];
    if (r.sats > 0) el.className = "funded";
    el.append(cell(String(i + 1).padStart(3, "0"), "idx"), cell(r.address, "addr"), cell(r.kind, "", "Type"),
      cell(r.sats === null ? "-" : btc(r.sats), "num", "BTC"), cell(r.sats === null ? "-" : fmt(r.sats), "num", "Sats"), cell(st, cls, "Status"));
    return el;
  };
  const total = rows.reduce((a, r) => a + (r.sats ?? 0), 0);
  $("rows").replaceChildren(...rows.map(tr));
  $("empty").hidden = rows.length > 0;
  $("total-btc").textContent = btc(total);
  $("total-sats").textContent = fmt(total);
  $("count").textContent = fmt(rows.length);
  $("check").disabled = busy || !rows.length;
  $("check").textContent = busy ? "Checking..." : "Check balances";
  $("clear").disabled = busy || !rows.length;
}

const say = (msg, bad = false) => { $("report").textContent = msg; $("report").classList.toggle("bad", bad); };

async function add(text) {
  const p = await parseBulk(text);
  const room = MAX_ADDRESSES - rows.length;
  const fresh = p.valid.filter((v) => !rows.some((r) => r.address === v.address));
  const take = fresh.slice(0, Math.max(room, 0));
  rows = rows.concat(take.map((v) => ({ ...v, sats: null, error: null })));
  const parts = [`${take.length} added`];
  const dup = p.duplicates + p.valid.length - fresh.length;
  if (dup) parts.push(`${dup} duplicate${dup > 1 ? "s" : ""} skipped`);
  if (fresh.length > take.length) parts.push(`limit of ${MAX_ADDRESSES} reached (use the desktop app for more)`);
  if (p.invalid.length) parts.push(`${p.invalid.length} invalid: ${p.invalid.slice(0, 5).join(", ")}${p.invalid.length > 5 ? " ..." : ""}`);
  say(parts.join(" · "), p.invalid.length > 0 && !take.length);
  render();
}

const TIMEOUT_MS = 10000;

/** mempool.space address object -> sats, or null when any field is missing or not a finite number. */
function parseBalance(v) {
  const net = (k) => v?.[k]?.funded_txo_sum - v?.[k]?.spent_txo_sum;
  const sats = net("chain_stats") + net("mempool_stats");
  return Number.isFinite(sats) ? Math.max(sats, 0) : null;
}

async function lookup(address) {
  try {
    const res = await fetch(API + address, { signal: AbortSignal.timeout(TIMEOUT_MS) });
    if (!res.ok) return { error: res.status === 429 ? "rate limited, retry shortly" : `HTTP ${res.status}` };
    const sats = parseBalance(await res.json());
    return sats === null ? { error: "unexpected response" } : { sats };
  } catch (e) {
    return { error: e.name === "TimeoutError" ? "timed out" : e.name === "SyntaxError" ? "unexpected response" : "network error" };
  }
}

async function check() {
  busy = true;
  render();
  try {
    for (const r of rows) {
      Object.assign(r, { error: null }, await lookup(r.address));
      render();
      await new Promise((ok) => setTimeout(ok, GAP_MS));
    }
  } finally {
    busy = false;
    const bad = rows.filter((r) => r.error).length;
    say(bad ? `Done with ${bad} error${bad > 1 ? "s" : ""}. Press Check again to retry.` : "Done.", bad > 0);
    render();
  }
}

async function addFiles(files) {
  for (const f of files) {
    if (!/\.(txt|csv)$/i.test(f.name)) say(`${f.name}: only .txt and .csv files are supported`, true);
    else if (f.size > MAX_FILE) say(`${f.name}: file is larger than 5 MB`, true);
    else await add(await f.text());
  }
}

$("single").onsubmit = (e) => { e.preventDefault(); add($("one").value); $("one").value = ""; };
$("bulk").onsubmit = (e) => { e.preventDefault(); add($("many").value); $("many").value = ""; };
$("file").onchange = (e) => addFiles(e.target.files).then(() => (e.target.value = ""));
$("drop").onkeydown = (e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); $("file").click(); } };
for (const t of ["dragenter", "dragover"]) document.addEventListener(t, (e) => { e.preventDefault(); $("drop").classList.add("over"); });
document.addEventListener("dragleave", (e) => { if (!e.relatedTarget) $("drop").classList.remove("over"); });
document.addEventListener("drop", (e) => { e.preventDefault(); $("drop").classList.remove("over"); addFiles(e.dataTransfer.files); });
$("check").onclick = check;
$("clear").onclick = () => { rows = []; say(""); render(); };
render();
