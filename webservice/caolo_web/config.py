import os


class Config(object):
    SECRET_KEY = os.getenv("FLASK_SECRET_KEY", "asdasdasdasdas")
    SQLALCHEMY_DATABASE_URI = os.getenv(
        "DATABASE_URL", "postgres://postgres:almafa1@localhost/caolo")
    SQLALCHEMY_TRACK_MODIFICATIONS = False
    GOOGLE_OAUTH_CLIENT_ID = os.getenv("GOOGLE_OAUTH_CLIENT_ID")
    GOOGLE_OAUTH_CLIENT_SECRET = os.getenv("GOOGLE_OAUTH_CLIENT_SECRET")
    GOOGLE_OAUTH_LOGIN_REDIRECT = os.getenv("GOOGLE_OAUTH_LOGIN_REDIRECT")

    PREFERRED_URL_SCHEME = os.getenv("PREFERRED_URL_SCHEME", "http")
    ON_LOGIN_REDIRECT = os.getenv("ON_LOGIN_REDIRECT")
