# hack path for protobufs
import sys
from pathlib import Path
sys.path.append(str(Path(__file__).parent.parent / "build"))

import os, sys, json

from flask import Flask, request, jsonify, abort, redirect, url_for
from flask_cors import CORS
from twisted.python import log
from twisted.internet import reactor
from twisted.web.server import Site
from twisted.web.wsgi import WSGIResource
from autobahn.twisted.websocket import WebSocketServerFactory, WebSocketServerProtocol
from autobahn.twisted.resource import WebSocketResource, WSGIRootResource
from websocket import create_connection
from flask_migrate import Migrate
from werkzeug.middleware.proxy_fix import ProxyFix
from flask_dance.contrib.google import google
from flask_login import current_user, login_required

from .handler.script import script_bp
from .handler.auth import auth_bp
from .config import Config
from .model import db, login_manager
from .service import get_redis_client

app = Flask(__name__)

app.config.from_object(Config)

db.init_app(app)
migrate = Migrate(app, db)
login_manager.init_app(app)
app.wsgi_app = ProxyFix(app.wsgi_app, x_proto=1, x_host=1)

cors = CORS(
    app, resources={r"/*": {
        "origins": "*"
    }}, supports_credentials=True)

app.register_blueprint(script_bp)
app.register_blueprint(auth_bp)


@app.route("/")
@login_required
def index():
    log.msg(f"index, current_user: {current_user}")

    if Config.ON_LOGIN_REDIRECT:
        return redirect(Config.ON_LOGIN_REDIRECT)

    return f"Hello {current_user.email.split('@')[0]}"


@app.route("/myself")
@login_required
def myself():
    user = {
        "id": current_user.id,
        "email": current_user.email,
    }
    return jsonify(user)


class SimulationProtocol(WebSocketServerProtocol):
    """
    Stream the world state to the clients
    """
    done = True
    loop = None
    latency = 0

    def onOpen(self):
        log.msg("opened simulation comms")
        latency = os.getenv("TARGET_TICK_FREQUENCY_MS", "2000")
        self.latency = int(latency) / 1000
        self.done = False
        reactor.callLater(self.latency, self.send_world_state)

    def onClose(self, *args):
        super().onClose(*args)
        self.done = True

    def send_world_state(self):
        import world_pb2
        from google.protobuf.json_format import MessageToDict

        log.msg(f"Handling world_state")
        if self.done:
            return
        else:
            reactor.callLater(self.latency, self.send_world_state)

        redis_conn = get_redis_client()
        payload = redis_conn.get("WORLD_STATE")
        if payload:
            world_state = json.loads(payload)
            log.msg(f"sending world state")
            payload = {"WORLD_STATE": world_state}
            payload = json.dumps(payload)
            self.sendMessage(payload.encode('utf8'))


def main():

    HOST = os.getenv("HOST", "localhost")
    PORT = int(os.getenv("PORT", "5000"))
    WS_PROTOCOL = os.getenv("WS_PROTOCOL", "ws")
    EXTERNAL_PORT = int(os.getenv("EXTERNAL_PORT", PORT))

    log.startLogging(sys.stdout)

    # create a Twisted Web resource for our WebSocket server
    wsFactory = WebSocketServerFactory(
        f"{WS_PROTOCOL}://{HOST}:{PORT}", externalPort=EXTERNAL_PORT)
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
