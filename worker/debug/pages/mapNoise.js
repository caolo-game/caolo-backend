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
    const [q, setQ] = useState(1);
    const [r, setR] = useState(1);

    useEffect(() => {
        setLoading(true);
        generateNoise({
            room: { q, r },
            roomSize: 16,
        })
            .then((res) => {
                setNoise(res);
                setLoading(false);
            })
            .catch(console.error);
    }, [setNoise, setLoading, q, r]);

    return (
        <>
            <Header />
            <main>
                {loading ? (
                    "loading..."
                ) : (
                    <form onSubmit={(e) => e.preventDefault()}>
                        <input
                            type="number"
                            onChange={(e) => setQ(parseInt(e.target.value))}
                            value={q}
                        />
                        <input
                            type="number"
                            onChange={(e) => setR(parseInt(e.target.value))}
                            value={r}
                        />
                    </form>
                )}
                <div
                    style={{ maxWidth: "50%" }}
                    dangerouslySetInnerHTML={{ __html: noise }}
                />
            </main>{" "}
        </>
    );
}
