import requests
import time
import random
import smtplib
import ssl
import os
from pathlib import Path
from dotenv import load_dotenv
from colorama import Fore, Style, init

# Initialize colorama
init()

# Resolve all paths relative to THIS script's location, not the cwd,
# so the bot works from any directory.
SCRIPT_DIR = Path(__file__).resolve().parent

# Load environment variables from .env file
load_dotenv(SCRIPT_DIR / ".env")

addresses = []
with open(SCRIPT_DIR / "address.txt") as f:
    for line in f:
        addr = line.strip()
        if addr:
            addresses.append(addr)

# Email settings (fill these in securely)
EMAIL_ADDRESS = os.getenv("BTC_ALERT_EMAIL")
EMAIL_PASSWORD = os.getenv("BTC_ALERT_PASSWORD")
TO_EMAIL = os.getenv("BTC_ALERT_TO", EMAIL_ADDRESS)

# Telegram settings
TELEGRAM_BOT_TOKEN = os.getenv("TELEGRAM_BOT_TOKEN")
TELEGRAM_CHAT_ID = os.getenv("TELEGRAM_CHAT_ID")

if not EMAIL_ADDRESS or not EMAIL_PASSWORD:
    raise EnvironmentError("Missing BTC_ALERT_EMAIL or BTC_ALERT_PASSWORD environment variables.")

if not TELEGRAM_BOT_TOKEN or not TELEGRAM_CHAT_ID:
    raise EnvironmentError("Missing TELEGRAM_BOT_TOKEN or TELEGRAM_CHAT_ID environment variables.")

# Send an email alert
def email_alert(subject, body, to_email):
    try:
        context = ssl.create_default_context()
        with smtplib.SMTP_SSL("smtp.gmail.com", 465, context=context) as server:
            server.login(EMAIL_ADDRESS, EMAIL_PASSWORD)
            message = f"Subject: {subject}\n\n{body}"
            server.sendmail(EMAIL_ADDRESS, to_email, message)
    except Exception as e:
        print(f"Failed to send email: {e}")

# Send a Telegram alert
def send_telegram_alert(message):
    try:
        url = f"https://api.telegram.org/bot{TELEGRAM_BOT_TOKEN}/sendMessage"
        payload = {"chat_id": TELEGRAM_CHAT_ID, "text": message}
        requests.post(url, data=payload)
    except Exception as e:
        print(f"Failed to send Telegram message: {e}")

# Track previously seen addresses
seen_with_balance = set()
seen_file = SCRIPT_DIR / "seen_balances.txt"

if os.path.exists(seen_file):
    with open(seen_file, "r") as f:
        for line in f:
            seen_with_balance.add(line.strip())

# Initialize random module
random.seed(0)

while True:
    print(Fore.CYAN + "🔍" + "=" * 48 + "🔍" + Style.RESET_ALL)
    print(Fore.YELLOW + f"⏰ Scanning BTC addresses - {time.strftime('%Y-%m-%d %H:%M:%S')} ⏰" + Style.RESET_ALL)
    print(Fore.CYAN + "🔍" + "=" * 48 + "🔍" + Style.RESET_ALL)
    for address in addresses:
        print(Fore.MAGENTA + f"📡 Scanning address {address} ..." + Style.RESET_ALL, end=' ')
        try:
            response = requests.get("https://blockchain.info/balance?active=" + address, timeout=10)
            if response.status_code != 200:
                print(Fore.RED + f"❌ Error: HTTP {response.status_code}" + Style.RESET_ALL)
                continue
            data = response.json()
            if address not in data or "final_balance" not in data[address]:
                print(Fore.RED + "⚠️ Error: Unexpected API response format." + Style.RESET_ALL)
                continue
            balance = data[address]["final_balance"]
            if balance > 0:
                print(Fore.GREEN + f"💰 Balance: {balance} sats" + Style.RESET_ALL)
                with open(SCRIPT_DIR / "balance_log.txt", "a") as log:
                    log.write(f"{time.strftime('%Y-%m-%d %H:%M:%S')} - {address} - {balance} sats\n")
                if address not in seen_with_balance:
                    seen_with_balance.add(address)
                    with open(seen_file, "a") as f:
                        f.write(address + "\n")
                    subject = f"[BTC Alert] New Balance Detected for {address}"
                    body = f"Address: {address}\nBalance: {balance} sats"
                    email_alert(subject, body, TO_EMAIL)
                    send_telegram_alert(body)
            else:
                print(Fore.LIGHTBLACK_EX + "🕸️ No funds." + Style.RESET_ALL)
        except Exception as e:
            print(Fore.RED + f"💥 Error: {e}" + Style.RESET_ALL)
        cooldown = 1 + (2 * random.random())
        time.sleep(cooldown)
    print(Fore.BLUE + "⏳ Waiting 60 seconds before next scan...\n" + Style.RESET_ALL)
    time.sleep(60)
