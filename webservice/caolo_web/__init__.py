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

app.register_blueprint(script_bp)
app.register_blueprint(auth_bp)

db.init_app(app)
migrate = Migrate(app, db)
login_manager.init_app(app)
app.wsgi_app = ProxyFix(app.wsgi_app, x_proto=1, x_host=1)

cors = CORS(app, resources={r"/*": {"origins": "*"}})


@app.route("/")
@login_required
def index():
    log.msg(f"index, current_user: {current_user}")

    if Config.ON_LOGIN_REDIRECT:
        return redirect(Config.ON_LOGIN_REDIRECT)

    return f"Hello {current_user.email.split('@')[0]}"



class SimulationProtocol(WebSocketServerProtocol):
    """
    Stream the simulation to the clients
    """
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
