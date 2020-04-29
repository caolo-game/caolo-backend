[![CircleCI](https://circleci.com/gh/caolo-game/caolo-backend/tree/master.svg?style=svg)](https://circleci.com/gh/caolo-game/caolo-backend/tree/master)

## Prerequisites

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [Redis](https://redis.io/)
- [Protoc](https://developers.google.com/protocol-buffers/docs/downloads.html)
- [PostgeSQL](https://www.postgresql.org/)
- [Docker](https://www.docker.com/) (Optional)
- [minikube](https://kubernetes.io/docs/tasks/tools/install-minikube/) (Optional)
- [kubectl](https://kubernetes.io/docs/tasks/tools/install-kubectl/) (Optional)

## Setting up

```
git submodule init
git submodule update
```

## Running via [Docker](https://www.docker.com/)

```
make
```

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
  make protopy
  cd webservice
  pip install -r requirements.txt
  maturin develop
  # create database called `caolo`
  python manage.py db upgrade
  python app.py
  ```

## Adding a proto

- Add to `worker/build.rs`

## Deployment

### Deploying to [Heroku](https://heroku.com)

Create a new project. Setup the repository. Then `make deploy`

### Starting a Kubernetes cluster locally:

- Start minikube `minikube start`
- `kubectl apply -f manifests`
