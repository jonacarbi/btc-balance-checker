//! Persistent address list, seen set, balance log and settings.
use crate::{address::{parse_bulk, Kind}, api::Outcome, settings::Settings};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, io::Write, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub address: String,
    pub kind: Kind,
    pub sats: Option<u64>,
    pub error: Option<String>,
    pub checked: Option<u64>,
}

#[derive(Debug, Default, Serialize)]
pub struct AddReport {
    pub added: usize,
    pub duplicates: usize,
    pub invalid: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Applied {
    pub fresh: Vec<(String, u64)>,
    pub problems: Vec<String>,
}

pub struct Store {
    data_dir: PathBuf,
    cfg_dir: PathBuf,
    /// CLI with a file keeps its list in memory only.
    pub ephemeral: bool,
    pub entries: Vec<Entry>,
    pub seen: BTreeSet<String>,
    pub settings: Settings,
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Unix seconds -> "YYYY-MM-DD HH:MM:SS" (UTC), civil-from-days algorithm.
pub fn utc(ts: u64) -> String {
    let (days, rem) = ((ts / 86400) as i64, ts % 86400);
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}", rem / 3600, rem % 3600 / 60, rem % 60)
}

/// Writes to a fresh `create_new` temp file (never reuses or follows an existing path), mode 0600, then renames.
fn write_atomic(path: &Path, body: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let name = path.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
    let tmp = path.with_file_name(format!(".{name}.{}.{}.tmp", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_nanos())));
    let mut opts = fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut opts, 0o600);
    let result = opts.open(&tmp).and_then(|mut f| f.write_all(body.as_bytes()).and_then(|()| f.sync_all())).and_then(|()| fs::rename(&tmp, path));
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

impl Store {
    pub fn default_dirs() -> (PathBuf, PathBuf) {
        let name = "btc-balance-checker";
        let base = |d: Option<PathBuf>| d.unwrap_or_else(|| PathBuf::from(".")).join(name);
        (base(dirs::data_dir()), base(dirs::config_dir()))
    }

    pub fn load(data_dir: PathBuf, cfg_dir: PathBuf) -> Self {
        let read = |p: PathBuf| fs::read_to_string(p).unwrap_or_default();
        Self {
            entries: serde_json::from_str(&read(data_dir.join("addresses.json"))).unwrap_or_default(),
            seen: read(data_dir.join("seen_balances.txt")).lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect(),
            settings: serde_json::from_str(&read(cfg_dir.join("settings.json"))).unwrap_or_default(),
            data_dir,
            cfg_dir,
            ephemeral: false,
        }
    }

    pub fn save_entries(&self) -> std::io::Result<()> {
        if self.ephemeral {
            return Ok(());
        }
        write_atomic(&self.data_dir.join("addresses.json"), &serde_json::to_string(&self.entries)?)
    }

    pub fn save_settings(&self) -> std::io::Result<()> {
        write_atomic(&self.cfg_dir.join("settings.json"), &serde_json::to_string_pretty(&self.settings)?)
    }

    fn append(&self, file: &str, line: &str) -> std::io::Result<()> {
        fs::create_dir_all(&self.data_dir)?;
        let mut f = fs::OpenOptions::new().create(true).append(true).open(self.data_dir.join(file))?;
        writeln!(f, "{line}")
    }

    pub fn add_text(&mut self, text: &str) -> AddReport {
        let parsed = parse_bulk(text);
        let mut rep = AddReport { duplicates: parsed.duplicates, invalid: parsed.invalid, added: 0 };
        for (address, kind) in parsed.valid {
            if self.entries.iter().any(|e| e.address == address) {
                rep.duplicates += 1;
            } else {
                self.entries.push(Entry { address, kind, sats: None, error: None, checked: None });
                rep.added += 1;
            }
        }
        rep
    }

    pub fn remove(&mut self, address: &str) {
        self.entries.retain(|e| e.address != address);
    }

    pub fn total_sats(&self) -> u64 {
        self.entries.iter().filter_map(|e| e.sats).sum()
    }

    /// Record lookup results. `fresh` lists balances seen for the first time. Failed lookups keep the
    /// previous balance and record the error. Persistence failures are returned in `problems`, never dropped;
    /// a failed seen-set write still alerts (the seen set stays correct in memory) but warns it may repeat.
    pub fn apply(&mut self, results: &[(String, Outcome)]) -> Applied {
        let ts = now();
        let mut out = Applied::default();
        for (addr, res) in results {
            let Some(e) = self.entries.iter_mut().find(|e| &e.address == addr) else { continue };
            e.checked = Some(ts);
            match res {
                Ok(sats) => {
                    e.sats = Some(*sats);
                    e.error = None;
                    if *sats > 0 {
                        let log = self.append("balance_log.txt", &format!("{} - {addr} - {sats} sats", utc(ts)));
                        out.problems.extend(log.err().map(|e| format!("could not write balance log: {e}")));
                        if first_seen(&mut self.seen, addr, *sats) {
                            let saved = self.append("seen_balances.txt", addr);
                            out.problems.extend(saved.err().map(|e| format!("could not save seen set ({e}); {addr} may alert again after a restart")));
                            out.fresh.push((addr.clone(), *sats));
                        }
                    }
                }
                Err(msg) => e.error = Some(msg.clone()),
            }
        }
        out.problems.extend(self.save_entries().err().map(|e| format!("could not save address list: {e}")));
        out
    }

    /// "txt" -> one address per line; "csv" -> address,btc,sats,status,last_checked.
    pub fn export(&self, format: &str) -> String {
        if format != "csv" {
            return self.entries.iter().map(|e| format!("{}\n", e.address)).collect();
        }
        let mut out = String::from("address,btc,sats,status,last_checked_utc\n");
        for e in &self.entries {
            let (btc, sats) = e.sats.map_or((String::new(), String::new()), |s| (btc(s), s.to_string()));
            let status = e.error.as_deref().unwrap_or(if e.sats.is_some() { "ok" } else { "pending" });
            let last = e.checked.map(utc).unwrap_or_default();
            out += &format!("{},{btc},{sats},\"{status}\",{last}\n", e.address);
        }
        out
    }
}

pub fn btc(sats: u64) -> String {
    format!("{}.{:08}", sats / 100_000_000, sats % 100_000_000)
}

/// A balance alert fires once per address: the first time a positive balance is observed.
pub fn first_seen(seen: &mut BTreeSet<String>, addr: &str, sats: u64) -> bool {
    sats > 0 && seen.insert(addr.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
    const B: &str = "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy";

    fn tmp(tag: &str) -> Store {
        let d = std::env::temp_dir().join(format!("bbc-test-{tag}-{}", now()));
        let _ = fs::remove_dir_all(&d);
        Store::load(d.join("data"), d.join("cfg"))
    }

    #[test]
    fn first_seen_fires_once_and_only_for_positive() {
        let mut seen = BTreeSet::new();
        assert!(!first_seen(&mut seen, A, 0));
        assert!(first_seen(&mut seen, A, 10));
        assert!(!first_seen(&mut seen, A, 20));
        assert!(!first_seen(&mut seen, A, 0));
    }

    #[test]
    fn apply_alerts_once_across_scans_and_survives_reload() {
        let mut s = tmp("apply");
        assert_eq!(s.add_text(&format!("{A} {B} {A} bad")).added, 2);
        let r = vec![(A.to_string(), Ok(5)), (B.to_string(), Ok(0))];
        assert_eq!(s.apply(&r).fresh, vec![(A.to_string(), 5)]);
        assert!(s.apply(&r).fresh.is_empty());
        assert_eq!(s.total_sats(), 5);
        let again = Store::load(s.data_dir.clone(), s.cfg_dir.clone());
        assert_eq!(again.entries.len(), 2);
        assert!(again.seen.contains(A));
        assert!(fs::read_to_string(s.data_dir.join("balance_log.txt")).unwrap().contains("5 sats"));
    }

    #[test]
    fn errors_keep_previous_balance() {
        let mut s = tmp("err");
        s.add_text(A);
        s.apply(&[(A.to_string(), Ok(7))]);
        s.apply(&[(A.to_string(), Err("HTTP 429".into()))]);
        assert_eq!(s.entries[0].sats, Some(7));
        assert_eq!(s.entries[0].error.as_deref(), Some("HTTP 429"));
    }

    #[test]
    fn export_formats() {
        let mut s = tmp("exp");
        s.add_text(&format!("{A} {B}"));
        s.apply(&[(A.to_string(), Ok(150_000_000))]);
        assert_eq!(s.export("txt"), format!("{A}\n{B}\n"));
        assert!(s.export("csv").contains(&format!("{A},1.50000000,150000000,\"ok\",")));
        assert!(s.export("csv").contains(&format!("{B},,,\"pending\",")));
    }

    #[test]
    fn utc_and_btc_format() {
        assert_eq!(utc(0), "1970-01-01 00:00:00");
        assert_eq!(utc(1_700_000_000), "2023-11-14 22:13:20");
        assert_eq!(btc(1), "0.00000001");
    }

    #[cfg(unix)]
    #[test]
    fn settings_file_is_private() {
        use std::os::unix::fs::PermissionsExt;
        let s = tmp("perm");
        s.save_settings().unwrap();
        let mode = fs::metadata(s.cfg_dir.join("settings.json")).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn failed_persistence_is_reported_and_alert_still_fires() {
        let mut s = tmp("fail");
        let blocker = s.data_dir.parent().unwrap().join("not-a-dir");
        fs::create_dir_all(blocker.parent().unwrap()).unwrap();
        fs::write(&blocker, "x").unwrap();
        s.data_dir = blocker.join("data"); // create_dir_all under a file always fails
        s.add_text(A);
        let out = s.apply(&[(A.to_string(), Ok(9))]);
        assert_eq!(out.fresh, vec![(A.to_string(), 9)]);
        assert!(out.problems.iter().any(|p| p.contains("seen set") && p.contains("may alert again")), "{:?}", out.problems);
        assert!(out.problems.iter().any(|p| p.contains("address list")), "{:?}", out.problems);
        assert!(s.seen.contains(A));
    }

    #[cfg(unix)]
    #[test]
    fn write_atomic_ignores_existing_or_symlinked_temp_files() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let s = tmp("tmpfile");
        fs::create_dir_all(&s.cfg_dir).unwrap();
        let victim = s.cfg_dir.join("victim.txt");
        fs::write(&victim, "keep").unwrap();
        // the old fixed names an attacker could pre-create
        symlink(&victim, s.cfg_dir.join("settings.tmp")).unwrap();
        fs::write(s.cfg_dir.join(".settings.json.tmp"), "x").unwrap();
        fs::set_permissions(s.cfg_dir.join(".settings.json.tmp"), fs::Permissions::from_mode(0o666)).unwrap();
        s.save_settings().unwrap();
        assert_eq!(fs::read_to_string(&victim).unwrap(), "keep");
        let mode = fs::metadata(s.cfg_dir.join("settings.json")).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn ephemeral_store_never_writes_the_address_list() {
        let mut s = tmp("eph");
        s.ephemeral = true;
        s.add_text(A);
        s.save_entries().unwrap();
        assert!(!s.data_dir.join("addresses.json").exists());
    }
}
