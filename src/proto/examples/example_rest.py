#!/bin/python

from http.server import BaseHTTPRequestHandler, HTTPServer

hostName = "0.0.0.0"
serverPort = 8080


class MyServer(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-type", "application/json")
        self.end_headers()
        self.wfile.write(bytes(r'{"obj":{"id":1}}', "utf-8"))


if __name__ == "__main__":
    web_server = HTTPServer((hostName, serverPort), MyServer)

    try:
        web_server.serve_forever()
    except KeyboardInterrupt:
        pass

    web_server.server_close()
