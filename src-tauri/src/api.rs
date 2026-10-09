//! Balance lookups: batched blockchain.info with per-address mempool.space fallback.
use serde_json::Value;
use std::{cell::Cell, collections::HashMap, thread::sleep, time::Duration};

const BLOCKCHAIN_INFO: &str = "https://blockchain.info/balance?active=";
const MEMPOOL: &str = "https://mempool.space/api/address/";
const BATCH: usize = 50;
const BATCH_GAP: Duration = Duration::from_millis(1200);
const FALLBACK_GAP: Duration = Duration::from_millis(400);
const MAX_BACKOFF: Duration = Duration::from_secs(60);

pub type Outcome = Result<u64, String>;

/// `{ "<addr>": { "final_balance": n, ... } }` -> address -> sats.
pub fn parse_blockchain_info(v: &Value) -> HashMap<String, u64> {
    let Some(map) = v.as_object() else { return HashMap::new() };
    map.iter().filter_map(|(a, e)| Some((a.clone(), e.get("final_balance")?.as_u64()?))).collect()
}

/// mempool.space address object -> confirmed + unconfirmed balance in sats.
pub fn parse_mempool(v: &Value) -> Option<u64> {
    let net = |k: &str| -> Option<i64> {
        let s = v.get(k)?;
        Some(s.get("funded_txo_sum")?.as_i64()? - s.get("spent_txo_sum")?.as_i64()?)
    };
    Some((net("chain_stats")? + net("mempool_stats")?).max(0) as u64)
}

/// HTTP client that slows every later request after a 429.
struct Client {
    agent: ureq::Agent,
    /// Extra delay before each request; grows on 429, resets on success.
    backoff: Cell<Duration>,
}

/// Next backoff: the server's `Retry-After` seconds if given, else double the previous (min 2 s), capped.
fn next_backoff(retry_after: Option<&str>, prev: Duration) -> Duration {
    let asked = retry_after.and_then(|v| v.trim().parse::<u64>().ok()).map(Duration::from_secs);
    asked.unwrap_or_else(|| (prev * 2).max(Duration::from_secs(2))).min(MAX_BACKOFF)
}

impl Client {
    fn new() -> Self {
        let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(10)).user_agent("btc-balance-checker/2").build();
        Self { agent, backoff: Cell::new(Duration::ZERO) }
    }

    fn get_json(&self, url: &str) -> Result<Value, String> {
        sleep(self.backoff.get());
        match self.agent.get(url).call() {
            Ok(r) => {
                self.backoff.set(Duration::ZERO);
                r.into_json().map_err(|e| format!("bad JSON: {e}"))
            }
            Err(ureq::Error::Status(429, r)) => {
                self.backoff.set(next_backoff(r.header("Retry-After"), self.backoff.get()));
                Err(format!("HTTP 429 (rate limited, waiting {}s before the next request)", self.backoff.get().as_secs()))
            }
            Err(ureq::Error::Status(code, _)) => Err(format!("HTTP {code}")),
            Err(e) => Err(format!("network: {}", e.kind())),
        }
    }

    fn mempool_one(&self, addr: &str) -> Outcome {
        let v = self.get_json(&format!("{MEMPOOL}{addr}"))?;
        parse_mempool(&v).ok_or_else(|| "unexpected API response".to_string())
    }

    fn fetch_chunk(&self, chunk: &[String]) -> Vec<(String, Outcome)> {
        let batch = self.get_json(&format!("{BLOCKCHAIN_INFO}{}", chunk.join("%7C"))).map(|v| parse_blockchain_info(&v));
        let found = batch.unwrap_or_default();
        let mut fell_back = false;
        chunk
            .iter()
            .map(|a| match found.get(a) {
                Some(&sats) => (a.clone(), Ok(sats)),
                None => {
                    if std::mem::replace(&mut fell_back, true) {
                        sleep(FALLBACK_GAP);
                    }
                    (a.clone(), self.mempool_one(a))
                }
            })
            .collect()
    }
}

/// Look up every address, calling `on_chunk` after each batch so callers can show progress.
pub fn fetch_all(addrs: &[String], mut on_chunk: impl FnMut(&[(String, Outcome)])) {
    let client = Client::new();
    for (i, chunk) in addrs.chunks(BATCH).enumerate() {
        if i > 0 {
            sleep(BATCH_GAP);
        }
        on_chunk(&client.fetch_chunk(chunk));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_blockchain_info_batch() {
        let v = json!({"A": {"final_balance": 5, "n_tx": 1, "total_received": 5}, "B": {"final_balance": 0}, "C": {"nope": 1}});
        let m = parse_blockchain_info(&v);
        assert_eq!(m.get("A"), Some(&5));
        assert_eq!(m.get("B"), Some(&0));
        assert!(!m.contains_key("C"));
        assert!(parse_blockchain_info(&json!([1])).is_empty());
    }

    #[test]
    fn parses_mempool_confirmed_and_pending() {
        let v = json!({"chain_stats": {"funded_txo_sum": 1000, "spent_txo_sum": 400},
                       "mempool_stats": {"funded_txo_sum": 50, "spent_txo_sum": 0}});
        assert_eq!(parse_mempool(&v), Some(650));
    }

    #[test]
    fn mempool_never_negative_and_rejects_malformed() {
        let v = json!({"chain_stats": {"funded_txo_sum": 0, "spent_txo_sum": 0},
                       "mempool_stats": {"funded_txo_sum": 0, "spent_txo_sum": 10}});
        assert_eq!(parse_mempool(&v), Some(0));
        assert_eq!(parse_mempool(&json!({"error": "x"})), None);
    }

    #[test]
    fn malformed_responses_are_dropped_not_guessed() {
        let m = parse_blockchain_info(&json!({"A": {"final_balance": "5"}, "B": {"final_balance": -1}, "C": {"final_balance": null}, "D": 7, "E": {"final_balance": 1.5}}));
        assert!(m.is_empty(), "{m:?}");
        assert_eq!(parse_mempool(&json!({"chain_stats": {"funded_txo_sum": "1", "spent_txo_sum": 0}, "mempool_stats": {}})), None);
        assert_eq!(parse_mempool(&json!(null)), None);
    }

    #[test]
    fn backoff_honours_retry_after_else_doubles_with_cap() {
        let z = Duration::ZERO;
        assert_eq!(next_backoff(Some("7"), z), Duration::from_secs(7));
        assert_eq!(next_backoff(Some("9999"), z), MAX_BACKOFF);
        assert_eq!(next_backoff(None, z), Duration::from_secs(2));
        assert_eq!(next_backoff(Some("Wed, 21 Oct 2026 07:28:00 GMT"), Duration::from_secs(4)), Duration::from_secs(8));
        assert_eq!(next_backoff(None, Duration::from_secs(50)), MAX_BACKOFF);
    }

    #[test]
    fn a_429_slows_later_requests_and_success_resets() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for (i, conn) in listener.incoming().take(2).enumerate() {
                let mut c = conn.unwrap();
                let _ = c.read(&mut [0; 1024]);
                let reply = if i == 0 { "429 Too Many Requests\r\nRetry-After: 1" } else { "200 OK" };
                let _ = write!(c, "HTTP/1.1 {reply}\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}");
            }
        });
        let c = Client::new();
        assert!(c.get_json(&url).unwrap_err().contains("429"));
        assert_eq!(c.backoff.get(), Duration::from_secs(1));
        let t = std::time::Instant::now();
        assert!(c.get_json(&url).is_ok());
        assert!(t.elapsed() >= Duration::from_secs(1), "next request must wait out the backoff");
        assert_eq!(c.backoff.get(), Duration::ZERO);
    }
}
