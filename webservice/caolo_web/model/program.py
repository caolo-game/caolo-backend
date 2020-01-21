from . import db
from .user import User
from sqlalchemy.dialects.postgresql import JSON, UUID


class Program(db.Model):
    """
    Users' Cao-Lang Programs
    """
    id = db.Column(
        UUID, primary_key=True, server_default=db.text("gen_random_uuid()"))
    program = db.Column(JSON, nullable=False)
    compiled = db.Column(JSON, nullable=True)
    name = db.Column(db.String, nullable=True)

    user_id = db.Column(UUID, db.ForeignKey(User.id), nullable=False)
    user = db.relationship(User)
