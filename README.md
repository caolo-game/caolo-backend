## Prerequisites

### Native builds:

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [Redis](https://redis.io/)
- [Protoc](https://developers.google.com/protocol-buffers/docs/downloads.html)
- [PostgeSQL](https://www.postgresql.org/)
- diesel-cli `cargo install diesel_cli --no-default-features --features "postgres"`

### Docker builds:

- [Docker](https://www.docker.com/)
- [Make](https://www.gnu.org/software/make/) (Optional)
- [kubectl](https://kubernetes.io/docs/tasks/tools/install-kubectl/) (Optional)
- [MiniKube](https://kubernetes.io/docs/tasks/tools/install-minikube/) (Optional, for local deployments)

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

- Building via Docker
  ```
  make
  ```

## Deployment

### Deploying to [Heroku](https://heroku.com)

`make deploy-heroku app=<your heroku app name>`

### Deploying via kubectl:

`make deploy`

### Running locally via [MiniKube](https://kubernetes.io/docs/setup/learning-environment/minikube/)

```
minkube start
echo <your google_userid> > google_userid
echo <your google_secret> > google_secret
kubectl create secret generic google-creds --from-file=./google_userid --from-file=./google_secret
make deploy
```
