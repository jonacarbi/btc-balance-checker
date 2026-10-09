import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "tests",
  testMatch: "*.spec.js",
  use: { baseURL: "http://127.0.0.1:47310" },
  webServer: [
    { command: "node tests/serve.mjs site 47310", port: 47310, reuseExistingServer: true },
    { command: "node tests/serve.mjs src 47311", port: 47311, reuseExistingServer: true },
  ],
});
