from . import caolo_web as cw

from flask import Flask, request, jsonify, abort
import json

app = Flask(__name__)


@app.route('/compile', methods=["POST"])
def compile_script():
    content = request.json
    content = json.dumps(content)
    try:
        res = cw.compile(content)
        return jsonify({"status": "ok", "compilation_result": res})
    except ValueError as e:
        print("Error compiling:", e)
        abort(400, e)
