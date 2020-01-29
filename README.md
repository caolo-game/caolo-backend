[![CircleCI](https://circleci.com/gh/snorrwe/caolo-backend/tree/master.svg?style=svg)](https://circleci.com/gh/snorrwe/caolo-backend/tree/master)

## Prerequisites

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [Python](https://www.python.org/)
- [Redis](https://redis.io/)
- [Docker](https://www.docker.com/) (Optional)
- [PostgeSQL](https://www.postgresql.org/) (for the webservice)

## Running via [Docker](https://www.docker.com/)

```
make
```

Then open the client in your browser by visiting [http://localhost:3000](http://localhost:3000)

## Building and running

- Running the worker
    ```
    cargo run
    ```

- Running the test suite
    ```
    cargo test
    cargo bench  # run benchmarks
    cargo clippy # linter
    ```

- Running the web service
    ```
    cd webservice
    pip install -r requirements.txt
    maturin develop
    # create database called `caolo`
    python manager.py db upgrade
    python app.py
    ```

## Deployment

Deploying to [Heroku](https://heroku.com):

Create a new project. Setup the repository. Then `make deploy`
