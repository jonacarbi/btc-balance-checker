//! Telegram + SMTP alerts for first-seen balances.
use crate::{settings::Settings, store::btc};
use lettre::{message::Mailbox, transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use std::time::Duration;

pub fn message(addr: &str, sats: u64) -> (String, String) {
    (
        format!("[BTC Alert] New Balance Detected for {addr}"),
        format!("Address: {addr}\nBalance: {sats} sats ({} BTC)", btc(sats)),
    )
}

fn scrub(msg: String, secret: &str) -> String {
    if secret.is_empty() { msg } else { msg.replace(secret, "***") }
}

pub fn send_telegram(s: &Settings, text: &str) -> Result<(), String> {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", s.telegram_token);
    ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(10))
        .build()
        .post(&url)
        .send_form(&[("chat_id", &s.telegram_chat_id), ("text", text)])
        .map(|_| ())
        .map_err(|e| scrub(format!("telegram: {e}"), &s.telegram_token))
}

pub fn send_email(s: &Settings, subject: &str, body: &str) -> Result<(), String> {
    let to = if s.email_to.is_empty() { &s.email_from } else { &s.email_to };
    let mailbox = |a: &str| a.parse::<Mailbox>().map_err(|e| format!("email address: {e}"));
    let msg = Message::builder()
        .from(mailbox(&s.email_from)?)
        .to(mailbox(to)?)
        .subject(subject)
        .body(body.to_string())
        .map_err(|e| format!("email: {e}"))?;
    let builder = if s.smtp_port == 465 { SmtpTransport::relay(&s.smtp_host) } else { SmtpTransport::starttls_relay(&s.smtp_host) };
    builder
        .map_err(|e| format!("smtp: {e}"))?
        .port(s.smtp_port)
        .timeout(Some(Duration::from_secs(15)))
        .credentials(Credentials::new(s.email_from.clone(), s.email_password.clone()))
        .build()
        .send(&msg)
        .map(|_| ())
        .map_err(|e| scrub(format!("smtp: {e}"), &s.email_password))
}

/// Send via every configured channel; returns one error string per failed channel.
pub fn deliver(s: &Settings, subject: &str, body: &str) -> Vec<String> {
    let mut errs = Vec::new();
    if s.telegram_ready() {
        errs.extend(send_telegram(s, body).err());
    }
    if s.email_ready() {
        errs.extend(send_email(s, subject, body).err());
    }
    errs
}

pub fn notify(s: &Settings, addr: &str, sats: u64) -> Vec<String> {
    let (subject, body) = message(addr, sats);
    deliver(s, &subject, &body)
}

/// Settings-panel "Send test alert": like `deliver`, but an unconfigured setup is an error.
pub fn send_test(s: &Settings) -> Result<(), String> {
    if !s.telegram_ready() && !s.email_ready() {
        return Err("no alert channel configured: fill in Telegram or email settings and save first".into());
    }
    match deliver(s, "[BTC Alert] Test", "BTC Balance Checker: test alert").join(" | ") {
        e if e.is_empty() => Ok(()),
        e => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_names_address_and_amount() {
        let (subject, body) = message("1abc", 150_000_000);
        assert!(subject.contains("1abc"));
        assert!(body.contains("150000000 sats") && body.contains("1.50000000 BTC"));
    }

    #[test]
    fn secrets_are_scrubbed_from_errors() {
        assert_eq!(scrub("https://x/bot123:ABC/send failed".into(), "123:ABC"), "https://x/bot***/send failed");
        assert_eq!(scrub("keep".into(), ""), "keep");
    }

    #[test]
    fn test_alert_needs_a_channel() {
        assert!(send_test(&Settings::default()).unwrap_err().contains("no alert channel"));
    }

    #[test]
    fn notify_without_channels_is_a_noop() {
        assert!(notify(&Settings::default(), "1abc", 5).is_empty());
    }
}
