# Cao-Lo backend

This repository contains the back-end code of the game Cao-Lo.

Code layout:

```txt
|- cao-storage-derive/ # Derive macro for the storage of the simulation/
|- migrations/         # SQL schema migrations
|- simulation/         # Library for running the game world
|- web/                # Webservice bridging remote clients and the worker
|- worker/             # Executable code running the simulation
```

## Building via Docker

```
make all
```

### Deploying to [Heroku](https://heroku.com)

`make deploy-heroku app=<your heroku app name>`
