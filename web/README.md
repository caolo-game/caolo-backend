## Prerequisites

### Native builds:

- [Python]()
- [Cap'n Proto](https://capnproto.org/)
- [PostgeSQL](https://www.postgresql.org/)
- diesel-cli `cargo install diesel_cli --no-default-features --features "postgres"`
- [Redis]()

### Docker builds:

- [Docker](https://www.docker.com/)
- [Make](https://www.gnu.org/software/make/) (Optional)

## Setting up

```
diesel database setup

python -m venv env
source env/scripts/activate
pip install -r requirements.txt
```

## Building and running

- Running the web service

  ```
  uvicorn caoloweb.app:app --reload
  ```
