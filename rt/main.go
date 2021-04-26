package main

import (
	"context"
	"flag"
	"io"
	"log"
	"net/http"

	cao_world "github.com/caolo-game/cao-rt/cao_world_pb"
	"github.com/caolo-game/cao-rt/world"
	"google.golang.org/grpc"

	"github.com/caolo-game/cao-rt/ws"
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

func initTerrain(conn *grpc.ClientConn, hub *ws.GameStateHub) {
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
		rid := world.RoomId{
			Q: roomId.Q,
			R: roomId.R,
		}
		hub.Terrain[rid] = terrain
	}
}

func main() {
	flag.Parse()

	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure())

	log.Println("Starting")

	conn, err := grpc.Dial(*simAddr, opts...)
	if err != nil {
		log.Fatalf("failed to connect %v", err)
	}
	defer conn.Close()
	hub := ws.NewGameStateHub()

	go listenToWorld(conn, hub.WorldState)

	go hub.Run()

	initTerrain(conn, hub)

	http.HandleFunc("/object-stream", func(w http.ResponseWriter, r *http.Request) {
		ws.ServeWs(hub, w, r)
	})

	log.Printf("Init done. Listening on %s", *addr)
	log.Fatal(http.ListenAndServe(*addr, nil))
}
