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
    cargo clippy # linter
    ```

## Directory/Module structure

- `api`: Rust module that contains all types and helper methods to interact between wasm and the engine and also between the worker and the client
- `engine`: The game engine that's responsible for running the simulation
- `worker`: Application that runs the engine and communicates with the "outside"
- `example-bot`: An example bot written in Rust that plays the game
