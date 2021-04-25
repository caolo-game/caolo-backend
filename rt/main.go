package main

import (
	"context"
	"io"
	"log"

    cao_world "github.com/caolo-game/cao-rt/cao_world_pb"
	"google.golang.org/grpc"
)

func main() {
	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure())

	conn, err := grpc.Dial("localhost:50051", opts...)
	if err != nil {
		log.Fatalf("failed to connect %v", err)
	}
	defer conn.Close()

	var client = cao_world.NewWorldClient(conn)

	stream, err := client.Entities(context.Background(), &cao_world.Empty{})
	if err != nil {
		panic(err)
	}

	for {
		entitites, err := stream.Recv()
		if err == io.EOF {
			log.Println("Bai")
			break
		}
		if err != nil {
			log.Fatalf("%v.Etities = %v", client, err)
		}

		log.Printf("pog %d rooms w/ bots received", len(entitites.Bots))

	}

}
