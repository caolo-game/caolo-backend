import { useEffect, useState } from "react";
import { promisified } from "tauri/api/tauri";

import Header from "../components/Header";

async function generateNoise({ room, roomSize, seed }) {
    const res = await promisified({
        cmd: "mapNoise",
        room,
        room_radius: roomSize,
        seed,
    });

    console.log("win", res);

    return res;
}

export default function MapNoise() {
    const [noise, setNoise] = useState(null);
    const [loading, setLoading] = useState(false);
    const [seed, setSeed] = useState(null);

    const [roomId, _] = useState([15, 16]);
    const [size, setSize] = useState(25);

    useEffect(() => {
        const [q, r] = roomId;
        setLoading(true);
        generateNoise({
            room: { q, r },
            roomSize: size,
            seed,
        })
            .then((res) => {
                setNoise(res);
                setLoading(false);
            })
            .catch(console.error);
    }, [setNoise, setLoading, roomId, seed, size]);

    return (
        <>
            <Header />
            <main>
                <form
                    onSubmit={(e) => {
                        e.preventDefault();

                        let seed = Math.floor(Math.random() * 1000000);
                        setSeed(seed);
                    }}
                >
                    <div>
                        <span>Size: </span>
                        <input
                            type="number"
                            onChange={(e) => setSize(parseInt(e.target.value))}
                            value={size}
                        />
                    </div>
                    <div>
                        <span>Seed: </span>
                        {seed}
                    </div>
                    <div>
                        <input
                            type="submit"
                            value="Random room"
                            disabled={loading}
                        />
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
