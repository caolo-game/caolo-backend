import { useEffect, useState } from "react";
import { promisified } from "tauri/api/tauri";

import Header from "../components/Header";

async function generateNoise({ room, roomSize }) {
    const res = await promisified({
        cmd: "mapNoise",
        room,
        room_radius: roomSize,
    });

    console.log("win", res);

    return res;
}

export default function MapNoise() {
    const [noise, setNoise] = useState(null);
    const [loading, setLoading] = useState(false);

    const [roomId, setRoomId] = useState([15, 16]);
    const [size, setSize] = useState(25);

    useEffect(() => {
        const [q, r] = roomId;
        setLoading(true);
        generateNoise({
            room: { q, r },
            roomSize: size,
        })
            .then((res) => {
                setNoise(res);
                setLoading(false);
            })
            .catch(console.error);
    }, [setNoise, setLoading, roomId, size]);

    return (
        <>
            <Header />
            <main>
                <form
                    onSubmit={(e) => {
                        e.preventDefault();
                        let t = Math.random();
                        let q = 1225 * t;
                        t = Math.random();
                        let r = 1225 * t;

                        q = Math.floor(q);
                        r = Math.floor(r);

                        setRoomId([q, r]);
                    }}
                >
                    <div>
                        <span> Size</span>
                        <input
                            type="number"
                            onChange={(e) => setSize(parseInt(e.target.value))}
                            value={size}
                        />
                    </div>
                    <div>
                        <input type="submit" value="Random room" />
                    </div>
                </form>
                <div>
                    RoomId: {roomId[0]} {roomId[1]}
                </div>
                <div
                    style={{ maxWidth: "50%" }}
                    dangerouslySetInnerHTML={{ __html: noise }}
                />
            </main>{" "}
        </>
    );
}
