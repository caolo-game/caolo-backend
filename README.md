## Prerequisites

### Native builds:

- [Cap'n Proto](https://capnproto.org/)
- [PostgeSQL](https://www.postgresql.org/)
- diesel-cli `cargo install diesel_cli --no-default-features --features "postgres"`
- [Golang]()
- [Redis]()

### Docker builds:

- [Docker](https://www.docker.com/)
- [Make](https://www.gnu.org/software/make/) (Optional)

## Setting up

```
diesel database setup
```

## Building and running

- Running the web service

  ```
  go run main.go
  ```

- Building via Docker
  ```
  make web
  ```

## Deployment

### Setting up Auth0

**TBA**

### Deploying to [Heroku](https://heroku.com)

`make deploy-heroku app=<your heroku app name>`
