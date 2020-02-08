from . import db
from .user import User
from sqlalchemy.dialects.postgresql import UUID, BYTEA


class Command(db.Model):
    """
    Commands issued by users
    """
    id = db.Column(
        UUID, primary_key=True, server_default=db.text("gen_random_uuid()"))

    # protobuf serialized payload
    raw_payload = db.Column(BYTEA, nullable=False)
    status = db.Column(
        db.Enum(
            'sending',
            'pending',
            'failed_to_send',
            'processing_error',
            'done',
            name='command_status',
            create_type=False),
        nullable=False)

    user_id = db.Column(UUID, db.ForeignKey(User.id), nullable=False)
    user = db.relationship(User)
