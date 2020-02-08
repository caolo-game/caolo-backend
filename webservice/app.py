# hack path for protobufs
import sys
from pathlib import Path
sys.path.append(str(Path(__file__).parent / "build"))

from caolo_web import main, app

try:
    from dotenv import load_dotenv
    load_dotenv()
except ImportError:
    pass

main()
