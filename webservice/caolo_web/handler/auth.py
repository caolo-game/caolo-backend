from flask import Blueprint, request, jsonify, abort, flash
from flask_login import current_user, login_user
from twisted.python import log
from flask_dance.contrib.google import google
from flask_dance.consumer import oauth_authorized, oauth_error
from flask_dance.contrib.google import make_google_blueprint
from flask_dance.consumer.storage.sqla import SQLAlchemyStorage
from sqlalchemy.orm.exc import NoResultFound

from ..model import OAuth, User, db
from ..config import Config

auth_bp = make_google_blueprint(
    scope=["profile", "email", "openid"],
    redirect_url=Config.GOOGLE_OAUTH_LOGIN_REDIRECT)


@oauth_authorized.connect_via(auth_bp)
def authorize(blueprint, token):
    if not token:
        log.err("Login failed")
        return False

    resp = blueprint.session.get("/oauth2/v1/userinfo")
    if not resp.ok:
        log.err("Failed to fetch user info.", resp.text)
        return False

    info = resp.json()
    user_id = info["id"]

    query = OAuth.query.filter_by(
        provider=blueprint.name, provider_user_id=user_id)
    try:
        oauth = query.one()
    except NoResultFound:
        oauth = OAuth(
            provider=blueprint.name, provider_user_id=user_id, token=token)

    if oauth.user:
        login_user(oauth.user)

    else:
        user = User(email=info["email"])
        oauth.user = user
        db.session.add_all([user, oauth])
        db.session.commit()
        login_user(user)

    log.msg(f"Successfully signed in. {user.id}")

    return False
