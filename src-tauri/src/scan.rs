//! One full scan: fetch -> record -> alert on first-seen balances.
use crate::{alerts, api, settings::Settings, store::Store};
use std::sync::Mutex;

#[derive(Default, serde::Serialize)]
pub struct Summary {
    pub checked: usize,
    pub errors: usize,
    pub alerts: usize,
    /// Alert delivery and persistence failures, so unattended scans never fail silently.
    pub problems: Vec<String>,
}

fn record(store: &Mutex<Store>, chunk: &[(String, api::Outcome)], sum: &mut Summary, notify: impl Fn(&Settings, &str, u64) -> Vec<String>) {
    let (applied, settings) = {
        let mut s = store.lock().unwrap();
        (s.apply(chunk), s.settings.clone())
    };
    sum.checked += chunk.len();
    sum.errors += chunk.iter().filter(|(_, r)| r.is_err()).count();
    sum.problems.extend(applied.problems);
    for (addr, sats) in applied.fresh {
        sum.alerts += 1;
        sum.problems.extend(notify(&settings, &addr, sats).into_iter().map(|e| format!("alert failed: {e}")));
    }
}

/// `on_chunk` runs after each batch is recorded (no lock held) for progress output.
pub fn scan(store: &Mutex<Store>, mut on_chunk: impl FnMut(&[(String, api::Outcome)])) -> Summary {
    let addrs: Vec<String> = store.lock().unwrap().entries.iter().map(|e| e.address.clone()).collect();
    let mut sum = Summary::default();
    api::fetch_all(&addrs, |chunk| {
        record(store, chunk, &mut sum, alerts::notify);
        on_chunk(chunk);
    });
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";

    #[test]
    fn alert_and_persistence_failures_surface_in_the_summary() {
        let dir = std::env::temp_dir().join(format!("bbc-scan-{}", crate::store::now()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("file"), "x").unwrap();
        let mut st = Store::load(dir.join("file").join("data"), dir.join("cfg")); // unwritable data dir
        st.add_text(A);
        let store = Mutex::new(st);
        let mut sum = Summary::default();
        record(&store, &[(A.to_string(), Ok(5))], &mut sum, |_, _, _| vec!["smtp: refused".into()]);
        assert_eq!((sum.checked, sum.alerts), (1, 1));
        assert!(sum.problems.iter().any(|p| p == "alert failed: smtp: refused"), "{:?}", sum.problems);
        assert!(sum.problems.iter().any(|p| p.contains("seen set")), "{:?}", sum.problems);
    }
}
