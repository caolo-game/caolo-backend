from flask_sqlalchemy import SQLAlchemy
db = SQLAlchemy()

from .user import *
from .program import *
