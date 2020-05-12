## Prerequisites

### Native builds:

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [NodeJs](https://nodejs.org/en/)
- [Redis](https://redis.io/)
- [Protoc](https://developers.google.com/protocol-buffers/docs/downloads.html)
- [PostgeSQL](https://www.postgresql.org/)
- neon_cli `npm install -g neon-cli`
- diesel-cli `cargo install diesel_cli --no-default-features --features "postgres"`

### Docker builds:

- [Docker](https://www.docker.com/)
- [kubectl](https://kubernetes.io/docs/tasks/tools/install-kubectl/)
- [minikube](https://kubernetes.io/docs/tasks/tools/install-minikube/) (Optional, for local deployments)

## Setting up

```
git submodule init
git submodule update
diesel database setup
```

## Building and running

- Running the worker

  ```
  cd worker
  cargo run
  ```

- Running the web service

  ```
  cd web
  yarn
  yarn start
  ```

## Deployment

### Deploying to [Heroku](https://heroku.com)

`make deploy-heroku app=<your heroku app name>`

### Deploying via kubectl:

`make deploy`
