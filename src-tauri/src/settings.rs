//! Alert settings. Stored in the OS config dir (0600 on unix); never hardcoded.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub telegram_token: String,
    pub telegram_chat_id: String,
    pub email_from: String,
    pub email_password: String,
    pub email_to: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub interval_secs: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            telegram_token: String::new(),
            telegram_chat_id: String::new(),
            email_from: String::new(),
            email_password: String::new(),
            email_to: String::new(),
            smtp_host: "smtp.gmail.com".into(),
            smtp_port: 465,
            interval_secs: 60,
        }
    }
}

pub const MIN_INTERVAL: u64 = 10;

impl Settings {
    pub fn telegram_ready(&self) -> bool {
        !self.telegram_token.is_empty() && !self.telegram_chat_id.is_empty()
    }
    pub fn email_ready(&self) -> bool {
        !self.email_from.is_empty() && !self.email_password.is_empty() && !self.smtp_host.is_empty()
    }
    pub fn interval(&self) -> u64 {
        self.interval_secs.max(MIN_INTERVAL)
    }

    /// Secrets blanked for the UI; `*_set` flags say whether one is stored.
    pub fn redacted(&self) -> serde_json::Value {
        let mut v = serde_json::to_value(self).unwrap_or_default();
        v["telegram_token_set"] = (!self.telegram_token.is_empty()).into();
        v["email_password_set"] = (!self.email_password.is_empty()).into();
        v["telegram_token"] = "".into();
        v["email_password"] = "".into();
        v
    }

    /// Apply an update from the UI; blank secrets keep the stored value.
    pub fn update(&mut self, mut new: Settings) {
        if new.telegram_token.is_empty() {
            new.telegram_token = std::mem::take(&mut self.telegram_token);
        }
        if new.email_password.is_empty() {
            new.email_password = std::mem::take(&mut self.email_password);
        }
        new.interval_secs = new.interval();
        *self = new;
    }

    /// Fill empty fields from legacy `.env` keys (KEY=VALUE lines).
    pub fn merge_env(&mut self, text: &str) {
        for line in text.lines() {
            let Some((k, v)) = line.trim().split_once('=') else { continue };
            let v = v.trim().trim_matches(|c| c == '"' || c == '\'').to_string();
            let slot = match k.trim() {
                "BTC_ALERT_EMAIL" => &mut self.email_from,
                "BTC_ALERT_PASSWORD" => &mut self.email_password,
                "TELEGRAM_BOT_TOKEN" => &mut self.telegram_token,
                "TELEGRAM_CHAT_ID" => &mut self.telegram_chat_id,
                _ => continue,
            };
            if slot.is_empty() {
                *slot = v;
            }
        }
        if self.email_to.is_empty() {
            self.email_to = self.email_from.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_import_maps_legacy_keys_and_keeps_existing() {
        let mut s = Settings { telegram_chat_id: "keep".into(), ..Default::default() };
        s.merge_env("# c\nBTC_ALERT_EMAIL=me@x.com\nBTC_ALERT_PASSWORD=\"pw\"\nTELEGRAM_BOT_TOKEN=tok\nTELEGRAM_CHAT_ID=999\nOTHER=1");
        assert_eq!((s.email_from.as_str(), s.email_password.as_str()), ("me@x.com", "pw"));
        assert_eq!((s.telegram_token.as_str(), s.telegram_chat_id.as_str()), ("tok", "keep"));
        assert_eq!(s.email_to, "me@x.com");
    }

    #[test]
    fn blank_secret_on_update_keeps_stored_and_interval_is_clamped() {
        let mut s = Settings { telegram_token: "tok".into(), email_password: "pw".into(), ..Default::default() };
        s.update(Settings { interval_secs: 1, email_from: "a@b.c".into(), ..Default::default() });
        assert_eq!((s.telegram_token.as_str(), s.email_password.as_str()), ("tok", "pw"));
        assert_eq!(s.interval_secs, MIN_INTERVAL);
        let r = s.redacted();
        assert_eq!(r["telegram_token"], "");
        assert_eq!(r["telegram_token_set"], true);
    }
}
