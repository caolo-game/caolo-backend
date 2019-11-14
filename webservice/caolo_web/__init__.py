import caolo_web_lib as cw

from flask import Flask, request, jsonify, abort
from flask_cors import CORS

app = Flask(__name__)
cors = CORS(app, resources={r"/*": {"origins": "*"}})


@app.route('/compile', methods=["POST"])
def compile_script():

    content = request.get_data(as_text=True)
    try:
        _program = cw.compile(content)
        return "successful compilation"
    except ValueError as e:
        print("Error compiling:", e)
        abort(400, e)
