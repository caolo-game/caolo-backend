package main

import (
	"context"
	"flag"
	"io"
	"log"
	"net/http"
	"time"

	cao_world "github.com/caolo-game/cao-rt/cao_world_pb"
	"google.golang.org/grpc"
	"google.golang.org/grpc/backoff"
)

var addr = flag.String("addr", "localhost:8080", "http service address")
var simAddr = flag.String("simAddr", "localhost:50051", "address of the Simulation Service")

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

func initTerrain(conn *grpc.ClientConn, hub *GameStateHub) {
	client := cao_world.NewWorldClient(conn)

	roomList, err := client.GetRoomList(context.Background(), &cao_world.Empty{})
	if err != nil {
		log.Fatalf("Failed to query terrain %v", err)
	}

	for i := range roomList.RoomIds {
		roomId := roomList.RoomIds[i]
		terrain, err := client.GetRoomTerrain(context.Background(), roomId)
		if err != nil {
			log.Fatalf("Failed to query terrain of room %v: %v", roomId, err)
		}
		rid := RoomId{
			Q: roomId.Q,
			R: roomId.R,
		}
		hub.Terrain[rid] = terrain
	}
}

func main() {
	flag.Parse()

	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure(), grpc.WithConnectParams(grpc.ConnectParams{
		Backoff: backoff.Config{
			BaseDelay:  time.Second * 2,
			Multiplier: 1.2,
			Jitter:     0.4,
			MaxDelay:   time.Second * 5,
		},
		MinConnectTimeout: time.Second * 10,
	}))

	log.Println("Starting")

	conn, err := grpc.Dial(*simAddr, opts...)
	if err != nil {
		log.Fatalf("failed to connect %v", err)
	}
	defer conn.Close()
	hub := NewGameStateHub()

	go listenToWorld(conn, hub.WorldState)

	go hub.Run()

	initTerrain(conn, hub)

	http.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNoContent)
	})

	http.HandleFunc("/object-stream", func(w http.ResponseWriter, r *http.Request) {
		ServeWs(hub, w, r)
	})

	log.Printf("Init done. Listening on %s", *addr)
	log.Fatal(http.ListenAndServe(*addr, nil))
}
