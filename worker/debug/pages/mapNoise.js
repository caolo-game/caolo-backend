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
    const [size, setSize] = useState(25);

    useEffect(() => {
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
    }, [setNoise, setLoading, q, r, size]);

    return (
        <>
            <Header />
            <main>
                    <form onSubmit={(e) => e.preventDefault()}>
                        <input
                            type="number"
                            onChange={(e) => setQ(parseInt(e.target.value))}
                            value={q}
                            disabled={loading}
                        />
                        <input
                            type="number"
                            onChange={(e) => setR(parseInt(e.target.value))}
                            value={r}
                            disabled={loading}
                        />
                        <input
                            type="number"
                            onChange={(e) => setSize(parseInt(e.target.value))}
                            value={size}
                            disabled={loading}
                        />
                    </form>
                <div
                    style={{ maxWidth: "50%" }}
                    dangerouslySetInnerHTML={{ __html: noise }}
                />
            </main>{" "}
        </>
    );
}
