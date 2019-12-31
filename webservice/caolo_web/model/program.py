from . import db
from .user import User
from sqlalchemy.dialects.postgresql import JSON, UUID


class Program(db.Model):
    """
    Users' Cao-Lang Programs
    """
    id = db.Column(UUID, primary_key=True)
    ast = db.Column(JSON, nullable=False)

    user_id = db.Column(UUID, db.ForeignKey(User.id), nullable=False)
    user = db.relationship(User)
