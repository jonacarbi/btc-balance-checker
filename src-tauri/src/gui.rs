//! Tauri commands. All state lives in `Store`; the UI re-fetches after each change.
use crate::{scan::scan, settings::Settings, store::{AddReport, Entry, Store}};
use serde::Serialize;
use std::{sync::{atomic::{AtomicBool, AtomicU64, Ordering::SeqCst}, Arc, Mutex}, thread, time::Duration};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;

struct Gui {
    store: Mutex<Store>,
    scanning: AtomicBool,
    watch_gen: AtomicU64,
}
type G<'a> = State<'a, Arc<Gui>>;

#[derive(Serialize)]
struct View {
    entries: Vec<Entry>,
    total_sats: u64,
    scanning: bool,
    watching: bool,
}

fn view(g: &Gui) -> View {
    let s = g.store.lock().unwrap();
    View { entries: s.entries.clone(), total_sats: s.total_sats(), scanning: g.scanning.load(SeqCst), watching: g.watch_gen.load(SeqCst) % 2 == 1 }
}

fn io(e: std::io::Error) -> String {
    e.to_string()
}

#[tauri::command]
fn get_state(g: G) -> View {
    view(&g)
}

#[tauri::command]
fn add_text(g: G, text: String) -> Result<AddReport, String> {
    let mut s = g.store.lock().unwrap();
    let rep = s.add_text(&text);
    s.save_entries().map_err(io)?;
    Ok(rep)
}

#[tauri::command]
fn remove(g: G, address: String) -> Result<(), String> {
    let mut s = g.store.lock().unwrap();
    s.remove(&address);
    s.save_entries().map_err(io)
}

#[tauri::command]
fn clear(g: G) -> Result<(), String> {
    let mut s = g.store.lock().unwrap();
    s.entries.clear();
    s.save_entries().map_err(io)
}

/// Save dialog; the chosen extension (.csv or .txt) picks the format. `None` means cancelled.
#[tauri::command]
async fn export_list(app: AppHandle, g: G<'_>, format: String) -> Result<Option<String>, String> {
    let ext = if format == "csv" { "csv" } else { "txt" };
    let Some(path) = app.dialog().file().add_filter("Address list", &["txt", "csv"]).set_file_name(format!("btc-addresses.{ext}")).blocking_save_file() else {
        return Ok(None);
    };
    let path = path.into_path().map_err(|e| e.to_string())?;
    let csv = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("csv"));
    let text = g.store.lock().unwrap().export(if csv { "csv" } else { "txt" });
    std::fs::write(&path, text).map_err(io)?;
    Ok(Some(path.display().to_string()))
}

/// Sends a test message through the saved alert settings.
#[tauri::command]
async fn test_alert(g: G<'_>) -> Result<(), String> {
    let settings = g.store.lock().unwrap().settings.clone();
    tauri::async_runtime::spawn_blocking(move || crate::alerts::send_test(&settings)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_settings(g: G) -> serde_json::Value {
    g.store.lock().unwrap().settings.redacted()
}

#[tauri::command]
fn save_settings(g: G, settings: Settings) -> Result<(), String> {
    let mut s = g.store.lock().unwrap();
    s.settings.update(settings);
    s.save_settings().map_err(io)
}

#[tauri::command]
fn import_env(g: G, text: String) -> Result<serde_json::Value, String> {
    let mut s = g.store.lock().unwrap();
    s.settings.merge_env(&text);
    s.save_settings().map_err(io)?;
    Ok(s.settings.redacted())
}

fn run_scan(app: &AppHandle, g: &Gui) -> Result<crate::scan::Summary, String> {
    if g.scanning.swap(true, SeqCst) {
        return Err("a scan is already running".into());
    }
    let _ = app.emit("changed", ());
    let sum = scan(&g.store, |_| {
        let _ = app.emit("changed", ());
    });
    g.scanning.store(false, SeqCst);
    let _ = app.emit("changed", ());
    Ok(sum)
}

#[tauri::command]
async fn check_now(app: AppHandle, g: G<'_>) -> Result<crate::scan::Summary, String> {
    let g = g.inner().clone();
    tauri::async_runtime::spawn_blocking(move || run_scan(&app, &g)).await.map_err(|e| e.to_string())?
}

/// Odd generation = watching. Each toggle bumps it, which retires any older watcher thread.
#[tauri::command]
fn set_watch(app: AppHandle, g: G, on: bool) {
    let cur = g.watch_gen.load(SeqCst);
    let mine = if on == (cur % 2 == 1) { cur } else { cur + 1 };
    g.watch_gen.store(mine, SeqCst);
    let _ = app.emit("changed", ());
    if !on || mine == cur {
        return;
    }
    let g = g.inner().clone();
    thread::spawn(move || {
        while g.watch_gen.load(SeqCst) == mine {
            if let Ok(sum) = run_scan(&app, &g) {
                if !sum.problems.is_empty() {
                    let _ = app.emit("scan-problems", sum.problems);
                }
            }
            let wait = g.store.lock().unwrap().settings.interval();
            for _ in 0..wait {
                if g.watch_gen.load(SeqCst) != mine {
                    return;
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
    });
}

pub fn run() {
    let (data, cfg) = Store::default_dirs();
    let gui = Arc::new(Gui { store: Mutex::new(Store::load(data, cfg)), scanning: AtomicBool::new(false), watch_gen: AtomicU64::new(0) });
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .manage(gui)
        .invoke_handler(tauri::generate_handler![
            get_state, add_text, remove, clear, export_list, get_settings, save_settings, import_env, test_alert, check_now, set_watch
        ])
        .run(tauri::generate_context!())
        .expect("error while running BTC Balance Checker");
}
