from setuptools import setup, find_packages
from pathlib import Path
import os
import sys


HERE = Path(__file__).parent

# manifest.in should copy the protos dir into this directory
PROTO_DIR = Path(os.getenv("CAO_PROTOS_PATH", HERE / ".." / "protos"))
try:
    os.listdir(PROTO_DIR)
except FileNotFoundError:
    # try here
    PROTO_DIR = HERE / "protos"
    try:
        os.listdir(PROTO_DIR)
    except FileNotFoundError as err:
        print(
            "Failed to find caolo protos directory. Try setting the CAO_PROTOS_PATH environment variable",
            err,
            file=sys.stderr,
        )
        sys.exit(1)


print(f"Found protos at {PROTO_DIR}")

PROTOC = "protoc"  # protoc command

res = os.system(" ".join([PROTOC, "--version"]))
assert res == 0, "can't find protoc command. Make sure protoc is installed and in $PATH"


# produce python files from our proto files
for e in os.listdir(PROTO_DIR):
    if ".proto" in e:
        res = os.system(
            " ".join(
                [
                    PROTOC,
                    "-I",
                    str(PROTO_DIR),
                    "--python_out",
                    str(HERE / "caoloweb/protos"),
                    str(PROTO_DIR / e),
                ]
            )
        )
        assert res == 0, f"Failed to compile proto {e} to Python"

setup(
    name="caoloweb",
    version="0.1.0",
    package_dir={"": "."},
    install_requires=[
        "fastapi",
        "asyncpg",
        "uvicorn[standard]",
        "pydantic[email]",
        "protobuf",
        "aioredis",
        "python-multipart",
        "passlib[bcrypt]",
        "python-jose>=3.2.0",
    ],
)
