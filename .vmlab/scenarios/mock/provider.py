"""A stand-in OpenAI-compatible Provider for the Guest.

    provider.py PORT STATUS ANSWER DELAY

GET answers a Model list; POST holds DELAY seconds, then answers ANSWER, or,
with STATUS of 400 and up, refuses with ANSWER as the error's message. Every
request is printed as "METHOD PATH #N BASE64BODY", so a Scenario can read the
prompt the app sent. Bound to loopback: a listener on 0.0.0.0 makes macOS ask
about the firewall in a dialog nobody answers.
"""
import base64
import json
import sys
import time
from http.server import BaseHTTPRequestHandler, HTTPServer

PORT, STATUS, ANSWER, DELAY = int(sys.argv[1]), int(sys.argv[2]), sys.argv[3], float(sys.argv[4])
seen = 0


class Provider(BaseHTTPRequestHandler):
    def _send(self, body, status=200):
        raw = json.dumps(body).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def _log(self, body=b""):
        global seen
        seen += 1
        print(f"{self.command} {self.path} #{seen} {base64.b64encode(body).decode()}", flush=True)

    def do_GET(self):
        self._log()
        self._send({"data": [{"id": "mock-small"}, {"id": "mock-large"}]})

    def do_POST(self):
        self._log(self.rfile.read(int(self.headers.get("Content-Length", 0))))
        time.sleep(DELAY)
        if STATUS >= 400:
            self._send({"error": {"message": ANSWER}}, STATUS)
        else:
            self._send({"choices": [{"message": {"content": ANSWER}}]})

    def log_message(self, *args):
        pass


HTTPServer(("127.0.0.1", PORT), Provider).serve_forever()
