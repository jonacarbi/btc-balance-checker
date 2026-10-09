const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);
const MAX_FILE = 5 * 1024 * 1024;
const KIND = { P2pkh: "P2PKH", P2sh: "P2SH", Bech32: "Bech32", Bech32m: "Taproot" };

const btc = (s) => `${Math.floor(s / 1e8)}.${String(s % 1e8).padStart(8, "0")}`;
const fmt = (n) => n.toLocaleString("en-US");
const cell = (tag, text, cls, label) => {
  const el = document.createElement(tag);
  el.textContent = text;
  if (cls) el.className = cls;
  if (label) el.dataset.label = label;
  return el;
};

function status(e) {
  if (e.error) return ["Error: " + e.error, "status-err"];
  if (e.sats === null) return ["Pending", "status-wait"];
  return e.sats > 0 ? ["Funded", "status-ok"] : ["Empty", "status-wait"];
}

function row(e, i) {
  const tr = document.createElement("tr");
  if (e.sats > 0) tr.className = "funded";
  const [st, cls] = status(e);
  const when = e.checked ? new Date(e.checked * 1000).toLocaleString() : "never";
  const rm = cell("button", "Remove", "btn quiet");
  rm.type = "button";
  rm.setAttribute("aria-label", `Remove ${e.address}`);
  rm.onclick = () => act(() => invoke("remove", { address: e.address }));
  const last = document.createElement("td");
  last.append(rm);
  tr.append(
    cell("td", String(i + 1).padStart(3, "0"), "idx"),
    cell("td", e.address, "addr"),
    cell("td", KIND[e.kind] || e.kind, "", "Type"),
    cell("td", e.sats === null ? "-" : btc(e.sats), "num", "BTC"),
    cell("td", e.sats === null ? "-" : fmt(e.sats), "num", "Sats"),
    cell("td", st, cls, "Status"),
    cell("td", when, "", "Checked"),
    last,
  );
  return tr;
}

let state = { entries: [], scanning: false, watching: false };

function render(s) {
  state = s;
  $("rows").replaceChildren(...s.entries.map(row));
  $("empty").hidden = s.entries.length > 0;
  $("total-btc").textContent = btc(s.total_sats);
  $("total-sats").textContent = fmt(s.total_sats);
  $("count").textContent = fmt(s.entries.length);
  $("check").disabled = s.scanning || !s.entries.length;
  $("check").textContent = s.scanning ? "Checking..." : "Check now";
  $("watch").setAttribute("aria-pressed", String(s.watching));
  $("watch").textContent = `Watch: ${s.watching ? "on" : "off"}`;
  document.body.classList.toggle("busy", s.scanning);
  for (const id of ["clear", "export-txt", "export-csv"]) $(id).disabled = !s.entries.length;
}

const refresh = () => invoke("get_state").then(render);
const say = (msg, bad = false, id = "report") => {
  $(id).textContent = msg;
  $(id).classList.toggle("bad", bad);
};
async function act(fn) {
  try {
    await fn();
  } catch (e) {
    say(String(e), true);
  }
  refresh();
}

function describe(r) {
  const parts = [`${r.added} added`];
  if (r.duplicates) parts.push(`${r.duplicates} duplicate${r.duplicates > 1 ? "s" : ""} skipped`);
  if (r.invalid.length) parts.push(`${r.invalid.length} invalid: ${r.invalid.slice(0, 5).join(", ")}${r.invalid.length > 5 ? " ..." : ""}`);
  return parts.join(" · ");
}

const addText = (text) =>
  act(async () => {
    const r = await invoke("add_text", { text });
    say(describe(r), r.invalid.length > 0 && r.added === 0);
  });

async function addFiles(files) {
  for (const f of files) {
    if (!/\.(txt|csv)$/i.test(f.name)) say(`${f.name}: only .txt and .csv files are supported`, true);
    else if (f.size > MAX_FILE) say(`${f.name}: file is larger than 5 MB`, true);
    else await addText(await f.text());
  }
}

$("single").onsubmit = (ev) => {
  ev.preventDefault();
  addText($("one").value);
  $("one").value = "";
};
$("bulk").onsubmit = (ev) => {
  ev.preventDefault();
  addText($("many").value);
  $("many").value = "";
};
$("file").onchange = (ev) => addFiles(ev.target.files).then(() => (ev.target.value = ""));
$("drop").onkeydown = (ev) => {
  if (ev.key === "Enter" || ev.key === " ") {
    ev.preventDefault();
    $("file").click();
  }
};
$("drop").tabIndex = 0;
for (const t of ["dragenter", "dragover"]) document.addEventListener(t, (ev) => { ev.preventDefault(); $("drop").classList.add("over"); });
document.addEventListener("dragleave", (ev) => { if (!ev.relatedTarget) $("drop").classList.remove("over"); });
document.addEventListener("drop", (ev) => {
  ev.preventDefault();
  $("drop").classList.remove("over");
  addFiles(ev.dataTransfer.files);
});

$("check").onclick = () =>
  act(async () => {
    const s = await invoke("check_now");
    const failed = s.problems.length ? ` · ${s.problems.join("; ")}` : "";
    say(`Checked ${s.checked} · ${s.errors} errors · ${s.alerts} new alert${s.alerts === 1 ? "" : "s"}${failed}`, s.errors > 0 || !!failed);
  });
$("watch").onclick = () => act(() => invoke("set_watch", { on: !state.watching }));
$("clear").onclick = () => confirm("Remove all addresses from the list?") && act(() => invoke("clear"));
for (const f of ["txt", "csv"]) $("export-" + f).onclick = () => act(async () => {
    const path = await invoke("export_list", { format: f });
    if (path) say("Saved to " + path);
  });

// Settings
const form = $("settings-form");
function fillSettings(s) {
  for (const el of form.elements) if (el.name && el.name in s && el.type !== "password") el.value = s[el.name];
  $("tok-set").textContent = s.telegram_token_set ? "(stored)" : "";
  $("pw-set").textContent = s.email_password_set ? "(stored)" : "";
}
$("open-settings").onclick = async () => {
  fillSettings(await invoke("get_settings"));
  say("", false, "set-msg");
  $("settings").showModal();
};
$("close-settings").onclick = () => $("settings").close();
form.onsubmit = async (ev) => {
  ev.preventDefault();
  const settings = Object.fromEntries([...form.elements].filter((e) => e.name).map((e) => [e.name, e.type === "number" ? Number(e.value) : e.value]));
  try {
    await invoke("save_settings", { settings });
    for (const el of form.querySelectorAll("input[type=password]")) el.value = "";
    fillSettings(await invoke("get_settings"));
    say("Saved.", false, "set-msg");
  } catch (e) {
    say(String(e), true, "set-msg");
  }
};
$("test-alert").onclick = async () => {
  say("Sending...", false, "set-msg");
  try {
    await invoke("test_alert");
    say("Test alert sent.", false, "set-msg");
  } catch (e) {
    say(String(e), true, "set-msg");
  }
};
$("env-btn").onclick = () => $("env-file").click();
$("env-file").onchange = async (ev) => {
  const f = ev.target.files[0];
  if (!f) return;
  try {
    fillSettings(await invoke("import_env", { text: await f.text() }));
    say("Imported legacy keys.", false, "set-msg");
  } catch (e) {
    say(String(e), true, "set-msg");
  }
  ev.target.value = "";
};

listen("changed", refresh);
// Problems found by unattended (watch) scans: failed alerts or failed saves.
listen("scan-problems", (ev) => say(`Watch scan: ${ev.payload.join("; ")}`, true));
refresh();
