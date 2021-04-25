from pathlib import Path
import os
import sys

HERE = Path(__file__).parent

PROTO_DIR = Path(os.getenv("CAO_PROTOS_PATH", HERE / ".." / "protos"))
try:
    os.listdir(PROTO_DIR)
except FileNotFoundError:
    print(
        "Failed to find caolo protos directory. Try setting the CAO_PROTOS_PATH environment variable",
        err,
        file=sys.stderr,
    )
    sys.exit(1)

print(f"Found protos at {PROTO_DIR}")

for e in os.listdir(PROTO_DIR):
    if ".proto" in e:
        name = e.split(".proto")[0]
        name = f"{name}_pb"
        try:
            os.mkdir(HERE / name)
        except FileExistsError:
            pass
        res = os.system(
            f"protoc --go_out=plugins=grpc:./{name} --go_opt=paths=source_relative -I{PROTO_DIR} {PROTO_DIR/e}"
        )
        assert res == 0, f"Failed to compile proto {e} to Python"
