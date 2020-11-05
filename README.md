## Prerequisites

### Native builds:

- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- [AMQP (e.g. RabbitMQ)](hhttps://www.rabbitmq.com/)
- [Protoc](https://developers.google.com/protocol-buffers/docs/downloads.html)
- [PostgeSQL](https://www.postgresql.org/)
- diesel-cli `cargo install diesel_cli --no-default-features --features "postgres"`

### Docker builds:

- [Docker](https://www.docker.com/)
- [Make](https://www.gnu.org/software/make/) (Optional)

## Setting up

```
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

### Setting up Auth0

TBA

### Deploying to [Heroku](https://heroku.com)

`make deploy-heroku app=<your heroku app name>`
