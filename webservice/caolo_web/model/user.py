from flask_login import LoginManager, UserMixin
from flask_dance.consumer.storage.sqla import OAuthConsumerMixin
from . import db


class User(UserMixin, db.Model):
    id = db.Column(db.Integer, primary_key=True)
    email = db.Column(db.String(256), unique=True)


class OAuth(OAuthConsumerMixin, db.Model):
    provider_user_id = db.Column(db.String(256), unique=True, nullable=False)
    user_id = db.Column(db.Integer, db.ForeignKey(User.id), nullable=False)
    user = db.relationship(User)


LOGIN_MANAGER = LoginManager()
LOGIN_MANAGER.login_view = "google.login"


@LOGIN_MANAGER.user_loader
def load_user(user_id):
    return User.query.get(int(user_id))
