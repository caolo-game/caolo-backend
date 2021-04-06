import { useEffect, useState } from "react";
import { promisified } from "tauri/api/tauri";

import Header from "../components/Header";

const radius = 18;

async function generateWorld({ world_radius, room_radius }) {
    // list of json serialized rooms
    const res = await promisified({
        cmd: "generateWorld",
        world_radius,
        room_radius,
    });

    return res.map(JSON.parse);
}

export default function MapGen() {
    const [rooms, setRooms] = useState([]);
    const [loading, setLoading] = useState(false);
    useEffect(() => {
        setLoading(true);
        generateWorld({ room_radius: radius, world_radius: 1 })
            .then((res) => {
                setRooms(res);
                setLoading(false);
            })
            .catch(console.error);
    }, [setRooms, setLoading]);

    return (
        <>
            <Header />
            <main>
                {loading ? (
                    "loading..."
                ) : (
                    <button
                        onClick={() => {
                            setLoading(true);
                            generateWorld({
                                room_radius: radius,
                                world_radius: 1,
                            })
                                .then((res) => {
                                    setRooms(res);
                                    setLoading(false);
                                })
                                .catch(console.error);
                        }}
                    >
                        Regenerate
                    </button>
                )}

                {rooms.map((room, i) => {
                    return (
                        <div key={i}>
                            <h2>
                                Room: ({room.roomId[0]}, {room.roomId[1]})
                            </h2>
                            <div
                                style={{
                                    maxWidth: "50%",
                                    margin: "0 auto",
                                }}
                                dangerouslySetInnerHTML={{
                                    __html: room.payload,
                                }}
                            />
                        </div>
                    );
                })}
            </main>
        </>
    );
}
