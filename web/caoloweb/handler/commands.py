from typing import Dict, List, Tuple
from fastapi import APIRouter, Response, Query, Request
import json
from pydantic import BaseModel

router = APIRouter(prefix="/commands")
