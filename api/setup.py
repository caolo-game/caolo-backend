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


# produce python files from our proto files
for e in os.listdir(PROTO_DIR):
    if ".proto" in e:
        res = os.system(
            " ".join(
                [
                    sys.executable,
                    "-m",
                    "grpc_tools.protoc",
                    "-I",
                    str(PROTO_DIR),
                    "--python_out",
                    str(HERE / "caoloapi/protos"),
                    "--grpc_python_out",
                    str(HERE / "caoloapi/protos"),
                    str(PROTO_DIR / e),
                ]
            )
        )
        assert res == 0, f"Failed to compile proto {e} to Python"

setup(
    name="caoloapi",
    package_dir={"": "."},
    install_requires=[
        "fastapi",
        "asyncpg",
        "uvicorn[standard]",
        "pydantic[email]",
        "grpcio-tools",
        "protobuf",
        "aioredis",
        "python-multipart",
        "passlib[bcrypt]",
        "python-jose>=3.2.0",
        "cao-lang @ https://github.com/caolo-game/cao-lang/tarball/master#egg=cao-lang-0.1.1",
    ],
)
