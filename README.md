# Cao-Lo backend

This repository contains the back-end code of the game Cao-Lo.

Code layout:

```txt
|- migrations/          # SQL schema migrations
|- protos/              # Protobuf messages used in communications between web and worker services
|- api/                 # Webservice bridging remote clients and the worker
|- sim/
 |+ cao-storage-derive/ # Derive macro for the storage of the simulation/
 |+ simulation/         # Library for running the game world
 |+ worker/             # Executable code running the simulation and interfacing
```

## Building via Docker

```
make all
```

### Deploying to [Heroku](https://heroku.com)

`make deploy-heroku app=<your heroku app name>`
