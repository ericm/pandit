#!/bin/python

from http.server import BaseHTTPRequestHandler, HTTPServer
import json
from math import factorial
from time import sleep

hostName = "0.0.0.0"
serverPort = 8080


class MyServer(BaseHTTPRequestHandler):
    def do_POST(self):
        content_length = int(self.headers['Content-Length'])
        print(content_length)
        post_data = self.rfile.read(content_length) 
        print("POST")
        data = json.loads(post_data.decode('utf-8'))
        print(data)
        num = int(data['number'])
        f = factorial(num)
        sleep(5)
        self.send_response(200)
        self.send_header("Content-type", "application/json")
        self.end_headers()
        data = json.dumps({"response": f})
        self.wfile.write(bytes(data, "utf-8"))


if __name__ == "__main__":
    web_server = HTTPServer((hostName, serverPort), MyServer)

    try:
        web_server.serve_forever()
    except KeyboardInterrupt:
        pass

    web_server.server_close()