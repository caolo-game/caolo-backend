"""empty message

Revision ID: 7a447ff002de
Revises: 
Create Date: 2019-12-31 19:01:42.561992

"""
from alembic import op
import sqlalchemy as sa
import sqlalchemy_utils
from sqlalchemy.dialects import postgresql

# revision identifiers, used by Alembic.
revision = '7a447ff002de'
down_revision = None
branch_labels = None
depends_on = None


def upgrade():
    # ### commands auto generated by Alembic - please adjust! ###
    op.execute(sa.text(
        """
        CREATE EXTENSION IF NOT EXISTS "pgcrypto";
        """
    ))
    op.create_table(
        'user',
        sa.Column(
            'id',
            postgresql.UUID(),
            nullable=False,
            primary_key=True,
            server_default=sa.text("gen_random_uuid()")),
        sa.Column('email', sa.String(length=256), nullable=True),
        sa.PrimaryKeyConstraint('id'), sa.UniqueConstraint('email'))
    op.create_table(
        'flask_dance_oauth', sa.Column('id', sa.Integer(), nullable=False),
        sa.Column('provider', sa.String(length=50), nullable=False),
        sa.Column('created_at', sa.DateTime(), nullable=False),
        sa.Column(
            'token', sqlalchemy_utils.types.json.JSONType(), nullable=False),
        sa.Column('provider_user_id', sa.String(length=256), nullable=False),
        sa.Column('user_id', postgresql.UUID(), nullable=False),
        sa.ForeignKeyConstraint(
            ['user_id'],
            ['user.id'],
        ), sa.PrimaryKeyConstraint('id'),
        sa.UniqueConstraint('provider_user_id'))
    # ### end Alembic commands ###


def downgrade():
    # ### commands auto generated by Alembic - please adjust! ###
    op.drop_table('flask_dance_oauth')
    op.drop_table('user')
    # ### end Alembic commands ###