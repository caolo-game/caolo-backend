![](https://github.com/snorrwe/caolo-backend/workflows/Rust/badge.svg)

## Prerequisites

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [Python](https://www.python.org/)
- [Docker](https://www.docker.com/) (Optional)

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
    python app.py
    ```
