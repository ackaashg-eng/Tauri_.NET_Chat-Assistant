// Connect screen of the desktop app: remembers the server address and opens the server's screens.
(function () {
  const KEY = "spil.serverUrl";
  const form = document.getElementById("form");
  const input = document.getElementById("url");
  const button = document.getElementById("go");
  const msg = document.getElementById("msg");

  const store = {
    get() { try { return localStorage.getItem(KEY) || ""; } catch { return ""; } },
    set(v) { try { localStorage.setItem(KEY, v); } catch { /* ignore */ } },
  };

  // "192.168.1.25:8000" -> "http://192.168.1.25:8000"
  function normalise(text) {
    let t = text.trim().replace(/\/+$/, "");
    if (!t) return null;
    if (!/^https?:\/\//i.test(t)) t = "http://" + t;
    try {
      const u = new URL(t);
      return u.origin;
    } catch { return null; }
  }

  function show(text, kind) { msg.textContent = text; msg.className = "msg " + (kind || ""); }

  // Asks GET /api/health and checks it is really a SPIL Labs server.
  async function check(url) {
    const ctrl = new AbortController();
    const timer = setTimeout(() => ctrl.abort(), 6000);
    try {
      const res = await fetch(url + "/api/health", { signal: ctrl.signal, cache: "no-store" });
      if (!res.ok) return "The server answered with error " + res.status + ".";
      const body = await res.json().catch(() => ({}));
      return body.app === "spil-labs-assistant" ? null : "That address is not a SPIL Labs server.";
    } catch (e) {
      return e.name === "AbortError" ? "The server did not answer in time." : "Could not reach the server. Check the address and that the server is running.";
    } finally { clearTimeout(timer); }
  }

  async function connect(url) {
    button.disabled = true;
    show("Connecting to " + url + " …");
    const problem = await check(url);
    button.disabled = false;
    if (problem) { show(problem, "bad"); return false; }
    store.set(url);
    show("Connected. Opening…", "ok");
    location.replace(url + "/");
    return true;
  }

  form.addEventListener("submit", (e) => {
    e.preventDefault();
    const url = normalise(input.value);
    if (!url) { show("Please type an address like http://server-name:8000", "bad"); return; }
    connect(url);
  });

  // First start: empty form. Later starts: go straight to the saved server
  // (unless the user chose "Change server…", which adds ?change=1).
  const saved = store.get();
  input.value = saved;
  const changing = new URLSearchParams(location.search).has("change");
  if (saved && !changing) connect(saved);
  else input.focus();
})();
