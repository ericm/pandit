#!/bin/python

from http.server import BaseHTTPRequestHandler, HTTPServer
import json
from math import factorial
from time import sleep

hostName = "0.0.0.0"
serverPort = 8080


class MyServer(BaseHTTPRequestHandler):
    def do_POST(self):
        # content_length = int(self.headers['Content-Length'])
        # post_data = self.rfile.read_all(content_length) 
        data = json.load(self.rfile)
        if 'number' in data:
            num = int(data['number'])
            f = factorial(num)
            sleep(5)
            json.dump({"response": f}, self.wfile)


if __name__ == "__main__":
    web_server = HTTPServer((hostName, serverPort), MyServer)

    try:
        web_server.serve_forever()
    except KeyboardInterrupt:
        pass

    web_server.server_close()