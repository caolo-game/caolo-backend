from caolo_web import app
import os

app.run(host="0.0.0.0", port=os.getenv("PORT"), threaded=True)
