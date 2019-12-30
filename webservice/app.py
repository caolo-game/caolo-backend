from caolo_web import main, app

try:
    from dotenv import load_dotenv
    load_dotenv()
except ImportError:
    pass

main()
