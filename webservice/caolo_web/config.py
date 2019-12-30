import os


class Config(object):
    SECRET_KEY = os.getenv("FLASK_SECRET_KEY") or "asdasdasdasdas"
    SQLALCHEMY_DATABASE_URI = os.getenv(
        "DATABASE_URL") or "postgres://postgres:almafa1@localhost/caolo"
    SQLALCHEMY_TRACK_MODIFICATIONS = False
    GOOGLE_OAUTH_CLIENT_ID = os.getenv("GOOGLE_OAUTH_CLIENT_ID")
    GOOGLE_OAUTH_CLIENT_SECRET = os.getenv("GOOGLE_OAUTH_CLIENT_SECRET")
