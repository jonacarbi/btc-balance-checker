// Static links already point at the releases page; this upgrades them to direct downloads when the API answers.
const REPO = "jonacarbi/btc-balance-checker";

async function load() {
  const status = document.getElementById("dl-status");
  try {
    const res = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, { headers: { Accept: "application/vnd.github+json" }, signal: AbortSignal.timeout(8000) });
    if (!res.ok) throw new Error(String(res.status));
    const rel = await res.json();
    for (const a of document.querySelectorAll("[data-asset]")) {
      const asset = (rel.assets || []).find((x) => x.name.toLowerCase().endsWith(`.${a.dataset.asset.toLowerCase()}`));
      if (asset) a.href = asset.browser_download_url;
    }
    status.textContent = `Latest release: ${rel.tag_name}`;
  } catch {
    status.textContent = "Could not reach the GitHub API. These links open the releases page.";
  }
}

// highlight the build for this machine
const ua = navigator.userAgent;
const mine = /Windows/.test(ua) ? "windows" : /Mac/.test(ua) ? "macos" : /Linux|X11/.test(ua) ? "linux" : "";
document.querySelector(`[data-os="${mine}"]`)?.classList.add("mine");
load();
