[![CircleCI](https://circleci.com/gh/caolo-game/caolo-backend/tree/master.svg?style=svg)](https://circleci.com/gh/caolo-game/caolo-backend/tree/master)

## Prerequisites

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [NodeJs](https://nodejs.org/en/)
- neon_cli `npm install -g neon-cli`
- [Redis](https://redis.io/)
- [Protoc](https://developers.google.com/protocol-buffers/docs/downloads.html)
- [PostgeSQL](https://www.postgresql.org/)
- diesel-cli `cargo install diesel_cli --no-default-features --features "postgres"`

- [Docker](https://www.docker.com/) (Optional)
- [minikube](https://kubernetes.io/docs/tasks/tools/install-minikube/) (Optional)
- [kubectl](https://kubernetes.io/docs/tasks/tools/install-kubectl/) (Optional)

## Setting up

```
git submodule init
git submodule update
diesel database setup
```

## Building and running

- Running the worker

  ```
  cargo run --bin caolo-worker
  ```

- Running the web service

  ```
  cargo run --bin caolo-web
  ```

- Running the test suite

  ```
  cargo test
  cargo test --benches # do a test-run on the benchmarks
  cargo bench  # run benchmarks
  cargo clippy # linter
  ```

## Adding a proto

- Add to `worker/build.rs`

## Deployment

### Deploying to [Heroku](https://heroku.com)

Create a new project. Setup the repository. Then `make deploy`

### Starting a Kubernetes cluster locally:

- Start minikube `minikube start`
- `kubectl apply -f manifests`
