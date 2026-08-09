// sergeant-rs embedded dashboard — the "small vanilla JavaScript" of §29.
//
// Two jobs and no more. The page is already server-rendered; this only keeps
// it live:
//
//   1. append incoming journal events to the ticker, so liveness is visible
//      without a round trip;
//   2. when an event could change what the server rendered, re-fetch this
//      same URL and swap <main> — the server stays the single renderer, so
//      there is no second copy of the view logic here to drift.
//
// The connection is an EventSource against the daemon's existing SSE
// endpoint (`/v1/events/stream`), authenticated by the `token` query
// parameter because EventSource cannot set an Authorization header.

(function () {
  "use strict";

  var root = document.getElementById("dashboard");
  if (!root) return;

  var token = root.getAttribute("data-token") || "";
  var from = parseInt(root.getAttribute("data-from") || "0", 10) || 0;
  var workId = root.getAttribute("data-work") || "";
  var indicator = document.getElementById("live");
  var ticker = document.getElementById("ticker");

  function setLive(state, text) {
    if (!indicator) return;
    indicator.setAttribute("data-state", state);
    indicator.textContent = text;
  }

  // Events that can change the server-rendered body. Everything else is
  // ticker-only: a refetch per keep-alive would be a busy loop with extra
  // steps.
  function isMaterial(kind) {
    return (
      kind.indexOf("work.") === 0 ||
      kind.indexOf("stage.") === 0 ||
      kind.indexOf("execution.") === 0 ||
      kind.indexOf("surface.") === 0 ||
      kind.indexOf("usage.") === 0 ||
      kind.indexOf("conversation.") === 0 ||
      kind.indexOf("tool.") === 0
    );
  }

  var pending = null;
  function scheduleRefresh() {
    if (pending) return;
    pending = window.setTimeout(function () {
      pending = null;
      refresh();
    }, 250);
  }

  function refresh() {
    fetch(window.location.href, { headers: { Accept: "text/html" } })
      .then(function (response) {
        if (!response.ok) throw new Error("refresh failed: " + response.status);
        return response.text();
      })
      .then(function (html) {
        var parsed = new DOMParser().parseFromString(html, "text/html");
        var next = parsed.querySelector("main");
        var current = document.querySelector("main");
        if (next && current) current.replaceWith(next);
      })
      .catch(function () {
        // A failed refresh is not a failed page: the ticker keeps running and
        // the next material event tries again.
      });
  }

  function addToTicker(event) {
    if (!ticker) return;
    var li = document.createElement("li");
    var seq = document.createElement("span");
    seq.className = "seq";
    seq.textContent = String(event.seq);
    var kind = document.createElement("span");
    kind.className = "kind";
    kind.textContent = event.kind;
    var who = document.createElement("span");
    who.textContent = event.work_id || "";
    li.appendChild(seq);
    li.appendChild(kind);
    li.appendChild(who);
    ticker.insertBefore(li, ticker.firstChild);
    while (ticker.childNodes.length > 50) ticker.removeChild(ticker.lastChild);
  }

  var url =
    "/v1/events/stream?from=" +
    encodeURIComponent(from) +
    "&token=" +
    encodeURIComponent(token);
  var source = new EventSource(url);

  setLive("connecting", "connecting…");
  source.onopen = function () {
    setLive("open", "live");
  };
  source.onerror = function () {
    // Two different failures wear the same event. When the browser has given
    // up (a non-200 response — a rotated token, say — fails an EventSource
    // permanently), CLOSED means no reconnect is coming and the page below is
    // frozen at whatever it last rendered; saying "reconnecting…" there would
    // be a comfortable lie.
    if (source.readyState === EventSource.CLOSED) {
      setLive("closed", "disconnected — reopen with a fresh `sgt web` URL");
    } else {
      setLive("closed", "reconnecting…");
    }
  };

  // Frames are typed with the journal's event kind (the SSE `event:` field),
  // and EventSource cannot subscribe to "any named frame" — so every name has
  // to be registered. The list is the *server's*, handed over in `data-kinds`
  // from the same constant the daemon names its frames with; a copy kept here
  // would be a second vocabulary to keep in step, and the one that used to
  // live here was already five kinds out of date. `onmessage` stays for a
  // frame that carries no name at all.
  source.addEventListener("message", handle);
  (root.getAttribute("data-kinds") || "").split(/\s+/).forEach(function (kind) {
    if (kind) source.addEventListener(kind, handle);
  });

  function handle(message) {
    var event;
    try {
      event = JSON.parse(message.data);
    } catch (e) {
      return;
    }
    addToTicker(event);
    if (workId && event.work_id && event.work_id !== workId) return;
    if (isMaterial(event.kind)) scheduleRefresh();
  }
})();
