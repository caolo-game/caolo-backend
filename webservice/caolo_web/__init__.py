import os, sys

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

app = Flask(__name__)
app.config['SECRET_KEY'] = 'secret!'

cors = CORS(app, resources={r"/*": {"origins": "*"}})


@app.route('/compile', methods=["POST"])
def compile_script():

    content = request.get_data(as_text=True)
    try:
        _program = cw.compile(content)
        return "successful compilation"
    except ValueError as e:
        print("Error compiling:", e)
        abort(400, e)


class SimulationProtocol(WebSocketServerProtocol):
    done = True
    redis_conn = None

    def onOpen(self):
        import redis
        self.redis_conn = redis.Redis(
            host=os.getenv("REDIS_HOST", "localhost"),
            port=os.getenv("REDIS_PORT", "6379"),
            db=0)
        self.done = False
        reactor.callLater(0.2, self.send_world_state)

    def onClose(self, *args):
        super().onClose(*args)
        self.done = True

    def send_world_state(self):
        if self.done:
            return
        payload = self.redis_conn.get("WORLD_STATE")
        self.sendMessage(payload)
        reactor.callLater(0.2, self.send_world_state)


def main():

    HOST = os.getenv("HOST", "localhost")
    PORT = int(os.getenv("PORT", "5000"))
    WS_PROTOCOL = os.getenv("WS_PROTOCOL", "ws")

    log.startLogging(sys.stdout)

    # create a Twisted Web resource for our WebSocket server
    wsFactory = WebSocketServerFactory(f"{WS_PROTOCOL}://{HOST}:{PORT}")
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
