import { useEffect, useState } from "react";
import { promisified } from "tauri/api/tauri";

async function generateWorld({ world_radius, room_radius }) {
    const res = await promisified({
        cmd: "generateWorld",
        world_radius,
        room_radius,
    });

    return res.map(JSON.parse);
}

export default function Home() {
    const [rooms, setRooms] = useState([]);
    const [selected, setSelectedRoom] = useState(null);
    useEffect(() => {
        generateWorld({ room_radius: 16, world_radius: 1 })
            .then((res) => {
                setSelectedRoom(0);
                setRooms(res);
            })
            .catch(console.error);
    }, [setRooms, setSelectedRoom]);

    const room = rooms[selected];
    console.log(room);

    if (!room) {
        return "loading...";
    }

    return (
        <div>
            <button
                onClick={() => {
                    setRooms([]);
                    generateWorld({ room_radius: 16, world_radius: 1 })
                        .then((res) => {
                            setSelectedRoom(0);
                            setRooms(res);
                        })
                        .catch(console.error);
                }}
            >
                Regenerate
            </button>

            <h2>
                Room: ({room.roomId.q}, {room.roomId.r})
            </h2>
            <div dangerouslySetInnerHTML={{ __html: room.payload }} />
        </div>
    );
}
