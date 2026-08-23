#!/usr/bin/env python3
"""Minimal `opencode` stand-in for W3's serve-transport tests.

Answers the version/help probe grammar exactly like `StubOpencode` (bash)
does for run-json, and additionally implements just enough of `opencode
serve`'s HTTP+SSE surface -- basic auth, /doc, /event (SSE), POST /session,
POST /session/{id}/message, GET /session/{id}/message, POST
/session/{id}/abort, POST /session/{id}/permissions/{id}, POST
/question/{id}/reply -- to drive `src/backend/opencode_serve.rs` against a
real socket. Driven entirely by a JSON "plan" file named by
SGT_STUBSERVE_PLAN; the plan is data, this script is mechanism.
"""
import base64
import json
import os
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

VERSION = "1.18.19"
RUN_HELP = (
    "opencode run [message..]\n"
    "Options: -s, --session  -m, --model  --format  --agent  --auto  --dir  -f, --file"
)
TOP_HELP = (
    "Commands: opencode run [message..] | opencode export [sessionID] | opencode serve | "
    "opencode models"
)
SERVE_HELP = "Options: --port  --hostname  --pure  --log-level  --print-logs"

DEFAULT_DOC = {
    "info": {"version": "1.0.0"},
    "security": [],
    "components": {"securitySchemes": None},
    "paths": {
        "/session": {"post": {"operationId": "session.create"}},
        "/session/{sessionID}/message": {
            "post": {"operationId": "session.prompt"},
            "get": {"operationId": "session.messages"},
        },
        "/session/{sessionID}/abort": {"post": {"operationId": "session.abort"}},
        "/session/{sessionID}/permissions/{permissionID}": {
            "post": {"operationId": "permission.respond", "deprecated": True}
        },
        "/question/{requestID}/reply": {"post": {"operationId": "question.reply"}},
        "/event": {"get": {"operationId": "event.subscribe"}},
    },
}


def load_plan():
    path = os.environ.get("SGT_STUBSERVE_PLAN")
    if path and os.path.exists(path):
        with open(path) as f:
            return json.load(f)
    return {}


def do_export(session_id):
    plan = load_plan()
    exports = plan.get("exports", {})
    if session_id in exports:
        print(json.dumps(exports[session_id]))
        return
    sys.stderr.write(f"Error: Session not found: {session_id}")
    sys.exit(1)


def serve():
    plan = load_plan()
    env_dump = os.environ.get("SGT_STUBSERVE_ENV_DUMP")
    if env_dump:
        with open(env_dump, "w") as f:
            json.dump(dict(os.environ), f)
    password = os.environ.get("OPENCODE_SERVER_PASSWORD", "")
    state = {"turn_index": 0, "queue": [], "cond": threading.Condition(), "closed": False}

    def push_frames(frames):
        with state["cond"]:
            for frame in frames or []:
                state["queue"].append(frame)
            state["cond"].notify_all()

    class Handler(BaseHTTPRequestHandler):
        # HTTP/1.0: every response closes its own connection. Simpler and
        # more robust for a test stub than reasoning about keep-alive/
        # connection-pool reuse across `/doc`, `/session`, and the
        # long-lived `/event` stream sharing one `reqwest::blocking::Client`.
        protocol_version = "HTTP/1.0"

        def log_message(self, *_args):
            pass

        def _authed(self):
            auth = self.headers.get("Authorization", "")
            if not auth.startswith("Basic "):
                return False
            try:
                decoded = base64.b64decode(auth[6:]).decode()
            except Exception:
                return False
            user, _, pw = decoded.partition(":")
            return user == "opencode" and pw == password

        def _unauthorized(self):
            body = b""
            self.send_response(401)
            self.send_header("WWW-Authenticate", 'Basic realm="Secure Area"')
            self.send_header("Content-Length", "0")
            self.end_headers()
            self.wfile.write(body)

        def _json(self, obj, status=200):
            body = json.dumps(obj).encode()
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_GET(self):
            if not self._authed():
                self._unauthorized()
                return
            if self.path == "/doc":
                self._json(plan.get("doc", DEFAULT_DOC))
                return
            if self.path == "/event":
                self.send_response(200)
                self.send_header("Content-Type", "text/event-stream")
                self.end_headers()
                try:
                    self.wfile.write(b'data: {"type":"server.connected","properties":{}}\n\n')
                    self.wfile.flush()
                except Exception:
                    return
                idx = 0
                while True:
                    with state["cond"]:
                        while idx >= len(state["queue"]) and not state["closed"]:
                            state["cond"].wait(timeout=0.2)
                        pending = state["queue"][idx:]
                        idx = len(state["queue"])
                        closed = state["closed"]
                    for frame in pending:
                        try:
                            self.wfile.write(("data: " + json.dumps(frame) + "\n\n").encode())
                            self.wfile.flush()
                        except Exception:
                            return
                    if closed:
                        return
                return
            if self.path.startswith("/session/") and self.path.endswith("/message"):
                self._json(plan.get("messages_response", []))
                return
            self._json({"error": "not found"}, 404)

        def do_POST(self):
            if not self._authed():
                self._unauthorized()
                return
            length = int(self.headers.get("Content-Length", 0) or 0)
            _body = self.rfile.read(length) if length else b""
            if self.path == "/session":
                self._json({"id": plan.get("session_id", "ses_stub1")})
                return
            if self.path.endswith("/abort"):
                push_frames(plan.get("abort_sse_frames", []))
                self._json(plan.get("abort_response", True))
                return
            if "/permissions/" in self.path:
                push_frames(plan.get("permission_reply_sse_frames", []))
                self._json(plan.get("permission_reply_response", True))
                return
            if self.path.startswith("/question/"):
                push_frames(plan.get("question_reply_sse_frames", []))
                self._json(plan.get("question_reply_response", True))
                return
            if self.path.endswith("/message"):
                turns = plan.get("turns", [])
                idx = state["turn_index"]
                state["turn_index"] += 1
                entry = turns[idx] if idx < len(turns) else {"response": {"info": {}, "parts": []}}
                push_frames(entry.get("sse_frames", []))
                self._json(entry.get("response", {}), entry.get("response_status", 200))
                return
            self._json({"error": "not found"}, 404)

    httpd = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    port = httpd.server_address[1]
    print(f"opencode server listening on http://127.0.0.1:{port}", flush=True)
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        pass


def main():
    args = sys.argv[1:]
    if args[:1] == ["--version"]:
        print(VERSION)
        return
    if args[:2] == ["run", "--help"]:
        sys.stderr.write(RUN_HELP)
        return
    if args[:2] == ["serve", "--help"]:
        sys.stderr.write(SERVE_HELP)
        return
    if args[:1] == ["serve"]:
        serve()
        return
    if args[:1] == ["--help"]:
        sys.stderr.write(TOP_HELP)
        return
    if args[:1] == ["export"] and len(args) > 1:
        do_export(args[1])
        return
    sys.exit(1)


if __name__ == "__main__":
    main()
