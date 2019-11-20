import os, sys, json

import caolo_web_lib as cw

from flask import Flask, request, jsonify, abort
from flask_cors import CORS

from twisted.python import log
from twisted.internet import reactor
from twisted.web.server import Site
from twisted.web.wsgi import WSGIResource

from autobahn.twisted.websocket import WebSocketServerFactory, WebSocketServerProtocol
from autobahn.twisted.resource import WebSocketResource, WSGIRootResource

from websocket import create_connection

import redis

app = Flask(__name__)
app.config['SECRET_KEY'] = 'secret!'

cors = CORS(app, resources={r"/*": {"origins": "*"}})


def get_redis_client():
    return redis.Redis.from_url(
        os.getenv("REDIS_URL", "redis://localhost:6379/0"))


@app.route('/script/compile', methods=["POST"])
def compile_script():

    content = request.get_data(as_text=True)
    try:
        _program = cw.compile(content)
        return "successful compilation"
    except ValueError as e:
        print("Error compiling:", e)
        abort(400, e)


@app.route('/script/commit', methods=["POST"])
def upload_script():
    program = request.json
    redis_conn = get_redis_client()
    content = request.json
    content = json.dumps(content)
    redis_conn.set("PROGRAM", content)
    return "Ok"


class SimulationProtocol(WebSocketServerProtocol):
    done = True
    redis_conn = None

    def onOpen(self):
        self.redis_conn = get_redis_client()
        self.done = False
        reactor.callLater(0.2, self.send_world_state)

    def onClose(self, *args):
        super().onClose(*args)
        self.done = True

    def send_world_state(self):
        if self.done:
            return
        payload = self.redis_conn.get("WORLD_STATE")
        if payload:
            self.sendMessage(payload)
        reactor.callLater(0.2, self.send_world_state)


def main():

    HOST = os.getenv("HOST", "localhost")
    PORT = int(os.getenv("PORT", "5000"))
    WS_PROTOCOL = os.getenv("WS_PROTOCOL", "ws")
    EXTERNAL_PORT = int(os.getenv("EXTERNAL_PORT", PORT))

    log.startLogging(sys.stdout)

    # create a Twisted Web resource for our WebSocket server
    wsFactory = WebSocketServerFactory(f"{WS_PROTOCOL}://{HOST}:{PORT}", externalPort=EXTERNAL_PORT)
    wsFactory.protocol = SimulationProtocol
    wsResource = WebSocketResource(wsFactory)

    # create a Twisted Web WSGI resource for our Flask server
    wsgiResource = WSGIResource(reactor, reactor.getThreadPool(), app)

    # create a root resource serving everything via WSGI/Flask, but
    # the path "/ws" served by our WebSocket stuff
    rootResource = WSGIRootResource(wsgiResource, {b"simulation": wsResource})

    # create a Twisted Web Site and run everything
    site = Site(rootResource)

    reactor.listenTCP(PORT, site)
    reactor.run()
