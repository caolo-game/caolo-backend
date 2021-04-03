## Prerequisites

### Native builds:

-   [Python](https://python.org/)
-   [Protoc](https://grpc.io/docs/protoc-installation/)
-   [PostgeSQL](https://www.postgresql.org/)
-   diesel-cli `cargo install diesel_cli --no-default-features --features "postgres"`
-   [Redis](https://keydb.dev/)

### Docker builds:

-   [Docker](https://www.docker.com/)
-   [Make](https://www.gnu.org/software/make/) (Optional)

## Setting up

```
diesel database setup

python -m venv env
source env/scripts/activate
pip install -e.
```

## Running

-   Running the web service

    ```
    uvicorn caoloapi.app:app --reload
    ```

## OpenAPI

Visit `http[s]://<url>/docs`
