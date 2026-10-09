// Bitcoin mainnet address validation with real checksums (mirror of src-tauri/src/address.rs).
const B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const B32 = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";

async function sha256(bytes) {
  return new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
}

async function base58check(s) {
  let n = 0n;
  for (const c of s) {
    const i = B58.indexOf(c);
    if (i < 0) return null;
    n = n * 58n + BigInt(i);
  }
  const bytes = [];
  for (; n > 0n; n >>= 8n) bytes.unshift(Number(n & 255n));
  for (const c of s) { if (c !== "1") break; bytes.unshift(0); }
  if (bytes.length < 5) return null;
  const payload = Uint8Array.from(bytes.slice(0, -4));
  const sum = (await sha256(await sha256(payload))).slice(0, 4);
  return sum.every((b, i) => b === bytes[bytes.length - 4 + i]) ? payload : null;
}

function polymod(values) {
  const G = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];
  let chk = 1;
  for (const v of values) {
    const top = chk >>> 25;
    chk = ((chk & 0x1ffffff) << 5) ^ v;
    G.forEach((g, i) => { if ((top >>> i) & 1) chk ^= g; });
  }
  return chk >>> 0;
}

function convert5to8(data) {
  let acc = 0, bits = 0;
  const out = [];
  for (const v of data) {
    acc = (acc << 5) | v;
    bits += 5;
    if (bits >= 8) { bits -= 8; out.push((acc >> bits) & 255); acc &= (1 << bits) - 1; }
  }
  return bits < 5 && acc === 0 ? out : null; // leftover padding must be < 5 bits and zero
}

function segwit(raw) {
  if (raw !== raw.toLowerCase() && raw !== raw.toUpperCase()) return null;
  const s = raw.toLowerCase();
  const sep = s.lastIndexOf("1");
  if (sep !== 2 || s.slice(0, 2) !== "bc" || s.length < 14) return null;
  const data = [...s.slice(sep + 1)].map((c) => B32.indexOf(c));
  if (data.includes(-1)) return null;
  const hrp = [...("bc")].map((c) => c.charCodeAt(0));
  const chk = polymod([...hrp.map((c) => c >> 5), 0, ...hrp.map((c) => c & 31), ...data]);
  const version = data[0];
  const variant = chk === 1 ? "Bech32" : chk === 0x2bc830a3 ? "Bech32m" : null;
  const prog = convert5to8(data.slice(1, -6));
  if (!variant || !prog) return null;
  if (version === 0 && variant === "Bech32" && (prog.length === 20 || prog.length === 32)) return { address: s, kind: "Bech32" };
  if (version === 1 && variant === "Bech32m" && prog.length === 32) return { address: s, kind: "Taproot" };
  return null;
}

/** @returns {Promise<{address:string, kind:string}|null>} */
export async function validate(raw) {
  if (raw.length > 90) return null;
  if (/^bc1/i.test(raw)) return segwit(raw);
  const p = await base58check(raw);
  if (p?.length !== 21) return null;
  return p[0] === 0 ? { address: raw, kind: "P2PKH" } : p[0] === 5 ? { address: raw, kind: "P2SH" } : null;
}

/** Split on whitespace/comma/semicolon, validate, dedupe. */
export async function parseBulk(text) {
  const out = { valid: [], invalid: [], duplicates: 0 };
  const seen = new Set();
  const tokens = text.split(/[\s,;]+/).map((t) => t.replace(/^["']+|["']+$/g, "")).filter(Boolean);
  for (const t of tokens) {
    const v = await validate(t);
    if (!v) { if (!out.invalid.includes(t)) out.invalid.push(t); }
    else if (seen.has(v.address)) out.duplicates++;
    else { seen.add(v.address); out.valid.push(v); }
  }
  return out;
}
