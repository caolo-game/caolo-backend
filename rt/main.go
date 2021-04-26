package main

import (
	"context"
	"flag"
	"io"
	"log"
	"net/http"

	cao_world "github.com/caolo-game/cao-rt/cao_world_pb"
	"google.golang.org/grpc"

	"github.com/caolo-game/cao-rt/ws"
)

var addr = flag.String("addr", ":8080", "http service address")

func listenToWorld(conn *grpc.ClientConn, worldState chan *cao_world.RoomEntities) {
	client := cao_world.NewWorldClient(conn)

	for {
		stream, err := client.Entities(context.Background(), &cao_world.Empty{})
		if err != nil {
			panic(err)
		}

		for {
			entitites, err := stream.Recv()
			if err == io.EOF {
				log.Println("Bai")
				return
			}
			if err != nil {
				log.Printf("Error in %v.Entities = %v", client, err)
				break
			}

			worldState <- entitites
		}
		log.Print("Retrying connection")
	}
}

func main() {
	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure())

	log.Println("Starting")

	conn, err := grpc.Dial("localhost:50051", opts...)
	if err != nil {
		log.Fatalf("failed to connect %v", err)
	}
	defer conn.Close()
	hub := ws.NewGameStateHub()

	go listenToWorld(conn, hub.WorldState)

	flag.Parse()

	go hub.Run()

	http.HandleFunc("/object-stream", func(w http.ResponseWriter, r *http.Request) {
		ws.ServeWs(hub, w, r)
	})

	log.Fatal(http.ListenAndServe(*addr, nil))
}
