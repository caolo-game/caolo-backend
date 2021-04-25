## Requirements

- Python

```sh
$ export GO111MODULE=on  # Enable module mode
$ go get google.golang.org/protobuf/cmd/protoc-gen-go \
         google.golang.org/grpc/cmd/protoc-gen-go-grpc
```


## Generate protobuf impls

```sh
python protos.py
```
