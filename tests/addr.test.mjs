import { test } from "node:test";
import assert from "node:assert/strict";
import { parseBulk, validate } from "../site/app/addr.js";

const P2PKH = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
const P2SH = "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy";
const BECH = "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq";
const TAPROOT = "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr";

test("accepts all four address types", async () => {
  assert.equal((await validate(P2PKH)).kind, "P2PKH");
  assert.equal((await validate(P2SH)).kind, "P2SH");
  assert.equal((await validate(BECH)).kind, "Bech32");
  assert.equal((await validate(TAPROOT)).kind, "Taproot");
});

test("rejects bad checksums, wrong network, wrong variant, mixed case", async () => {
  for (const bad of [
    "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNb",
    "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdp",
    "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx",
    "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kemeawh",
    "bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqh2y7hd",
    "BC1" + BECH.slice(3),
    "hello",
    "",
  ]) assert.equal(await validate(bad), null, bad);
});

test("uppercase bech32 is canonicalised to lowercase", async () => {
  assert.equal((await validate(BECH.toUpperCase())).address, BECH);
});

test("bulk parsing splits, dedupes and reports invalid", async () => {
  const p = await parseBulk(`${P2PKH}, ${P2SH}\n"${BECH}";${P2PKH} nope\n${BECH} nope`);
  assert.equal(p.valid.length, 3);
  assert.equal(p.duplicates, 2);
  assert.deepEqual(p.invalid, ["nope"]);
});
