import caolo_web_lib as cw

from flask import Flask, request, jsonify, abort
import json

app = Flask(__name__)


@app.route('/compile', methods=["POST"])
def compile_script():

    content = request.get_data(as_text=True)
    try:
        _program = cw.compile(content)
        return "successful compilation"
    except ValueError as e:
        print("Error compiling:", e)
        abort(400, e)
