//! `--cli [file]`: legacy-style terminal watch loop.
use crate::{scan::scan, settings::Settings, store::{btc, now, utc, Store}};
use std::{fs, path::Path, sync::Mutex, thread::sleep, time::Duration};

const LEGACY_KEYS: [&str; 4] = ["BTC_ALERT_EMAIL", "BTC_ALERT_PASSWORD", "TELEGRAM_BOT_TOKEN", "TELEGRAM_CHAT_ID"];

fn paint(code: u8, s: &str) -> String {
    format!("\x1b[{code}m{s}\x1b[0m")
}

/// Settings fall back to env vars, then to a `.env` next to the address file.
fn fill_from_env(s: &mut Settings, file: Option<&Path>) {
    let env_text: String = LEGACY_KEYS.iter().filter_map(|k| Some(format!("{k}={}\n", std::env::var(k).ok()?))).collect();
    s.merge_env(&env_text);
    if let Some(dir) = file.and_then(Path::parent) {
        s.merge_env(&fs::read_to_string(dir.join(".env")).unwrap_or_default());
    }
}

/// `--cli [file] [--once]`. With a file, only that file is scanned and the GUI list is never read or written.
pub fn run(args: &[String]) -> Result<(), String> {
    let once = args.iter().any(|a| a == "--once");
    let file = args.iter().find(|a| a.as_str() != "--once").map(Path::new);
    let (data, cfg) = Store::default_dirs();
    let mut store = Store::load(data, cfg);
    if let Some(path) = file {
        let text = fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        store.entries.clear();
        store.ephemeral = true;
        let rep = store.add_text(&text);
        for bad in &rep.invalid {
            eprintln!("{}", paint(33, &format!("skipping invalid address: {bad}")));
        }
    }
    fill_from_env(&mut store.settings, file);
    if store.entries.is_empty() {
        return Err("no addresses: pass a file (btc-balance-checker --cli address.txt) or add some in the app".into());
    }
    let interval = store.settings.interval();
    let store = Mutex::new(store);
    loop {
        println!("{}", paint(36, &"=".repeat(50)));
        println!("{}", paint(33, &format!("Scanning BTC addresses - {} UTC", utc(now()))));
        println!("{}", paint(36, &"=".repeat(50)));
        let sum = scan(&store, |chunk| chunk.iter().for_each(print_result));
        for e in &sum.problems {
            println!("{}", paint(31, e));
        }
        if once {
            return Ok(());
        }
        println!("{}", paint(34, &format!("Waiting {interval} seconds before next scan...\n")));
        sleep(Duration::from_secs(interval));
    }
}

fn print_result((addr, res): &(String, crate::api::Outcome)) {
    print!("{} ", paint(35, &format!("Scanning address {addr} ...")));
    match res {
        Ok(0) => println!("{}", paint(90, "No funds.")),
        Ok(n) => println!("{}", paint(32, &format!("Balance: {n} sats ({} BTC)", btc(*n)))),
        Err(e) => println!("{}", paint(31, &format!("Error: {e}"))),
    }
}
