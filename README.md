![](https://github.com/snorrwe/caolo-backend/workflows/Rust/badge.svg)

## Prerequisites

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
    - Rust compiler version 1.38+

## Running via [Docker](https://www.docker.com/)

```
docker-compose up
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
