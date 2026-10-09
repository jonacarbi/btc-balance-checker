//! Bitcoin mainnet address validation (real checksum verification) and bulk parsing.
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    P2pkh,
    P2sh,
    Bech32,
    Bech32m,
}

/// Validate a mainnet address; returns its canonical form (bech32 lowercased) and kind.
pub fn validate(raw: &str) -> Result<(String, Kind), String> {
    if raw.len() > 90 {
        return Err("too long".into());
    }
    if raw.get(..3).is_some_and(|p| p.eq_ignore_ascii_case("bc1")) {
        return segwit(raw);
    }
    let bytes = bs58::decode(raw).with_check(None).into_vec().map_err(|_| "bad Base58 checksum".to_string())?;
    match (bytes.len(), bytes.first()) {
        (21, Some(0x00)) => Ok((raw.into(), Kind::P2pkh)),
        (21, Some(0x05)) => Ok((raw.into(), Kind::P2sh)),
        _ => Err("unknown address type".into()),
    }
}

fn segwit(raw: &str) -> Result<(String, Kind), String> {
    if raw != raw.to_ascii_lowercase() && raw != raw.to_ascii_uppercase() {
        return Err("mixed-case bech32".into());
    }
    let lower = raw.to_ascii_lowercase();
    let (hrp, version, program) = bech32::segwit::decode(&lower).map_err(|e| format!("bad bech32: {e}"))?;
    if hrp.as_str() != "bc" {
        return Err("not a mainnet address".into());
    }
    let kind = match (u8::from(version), program.len()) {
        (0, 20 | 32) => Kind::Bech32,
        (1, 32) => Kind::Bech32m,
        _ => return Err("unsupported witness version or program length".into()),
    };
    Ok((lower, kind))
}

#[derive(Debug, Default, PartialEq)]
pub struct Parsed {
    pub valid: Vec<(String, Kind)>,
    pub invalid: Vec<String>,
    pub duplicates: usize,
}

/// Split on whitespace, commas and semicolons; strip quotes; validate; dedupe (order kept).
pub fn parse_bulk(text: &str) -> Parsed {
    let mut out = Parsed::default();
    let mut seen = HashSet::new();
    let tokens = text.split(|c: char| c.is_whitespace() || c == ',' || c == ';');
    for tok in tokens.map(|t| t.trim_matches(|c| c == '"' || c == '\'')).filter(|t| !t.is_empty()) {
        match validate(tok) {
            Ok((addr, kind)) if seen.insert(addr.clone()) => out.valid.push((addr, kind)),
            Ok(_) => out.duplicates += 1,
            Err(_) if !out.invalid.iter().any(|i| i == tok) => out.invalid.push(tok.to_string()),
            Err(_) => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const P2PKH: &str = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
    const P2SH: &str = "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy";
    const BECH: &str = "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq";
    const TAPROOT: &str = "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr";

    #[test]
    fn accepts_all_four_types() {
        assert_eq!(validate(P2PKH).unwrap().1, Kind::P2pkh);
        assert_eq!(validate(P2SH).unwrap().1, Kind::P2sh);
        assert_eq!(validate(BECH).unwrap().1, Kind::Bech32);
        assert_eq!(validate(TAPROOT).unwrap().1, Kind::Bech32m);
    }

    #[test]
    fn rejects_bad_checksums() {
        assert!(validate("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNb").is_err());
        assert!(validate("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdp").is_err());
        assert!(validate("bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcs").is_err());
    }

    #[test]
    fn rejects_wrong_network_and_garbage() {
        assert!(validate("tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx").is_err());
        assert!(validate("mipcBbFg9gMiCh81Kj8tqqdgoZub1ZJRfn").is_err());
        assert!(validate("hello").is_err());
        assert!(validate("").is_err());
    }

    #[test]
    fn rejects_wrong_bech32_variant() {
        // v0 program encoded with the bech32m constant must fail (BIP350).
        assert!(validate("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kemeawh").is_err());
        // v1 program encoded as classic bech32 must fail.
        assert!(validate("bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqh2y7hd").is_err());
    }

    #[test]
    fn bech32_is_canonicalised_and_mixed_case_rejected() {
        assert_eq!(validate(&BECH.to_uppercase()).unwrap().0, BECH);
        let mixed = format!("BC1{}", &BECH[3..]);
        assert!(validate(&mixed).is_err());
    }

    #[test]
    fn bulk_splits_dedupes_and_reports_invalid() {
        let text = format!("{P2PKH}, {P2SH}\n\"{BECH}\";{P2PKH} nope\n{BECH} nope");
        let p = parse_bulk(&text);
        assert_eq!(p.valid.len(), 3);
        assert_eq!(p.duplicates, 2);
        assert_eq!(p.invalid, vec!["nope"]);
    }

    #[test]
    fn bulk_handles_empty_and_csv() {
        assert_eq!(parse_bulk(" \n ,, "), Parsed::default());
        let p = parse_bulk(&format!("address,label\r\n{P2PKH},genesis\r\n"));
        assert_eq!(p.valid.len(), 1);
        assert_eq!(p.invalid, vec!["address", "label", "genesis"]);
    }
}
