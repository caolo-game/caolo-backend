import { useEffect, useState } from "react";

import Header from "../components/Header";
import { promisified } from "tauri/api/tauri";

const SQRT_3 = Math.sqrt(3);

function axialToPixelPointy(size, { q, r }) {
    const x = size * (SQRT_3 * q + (SQRT_3 / 2.0) * r);
    const y = size * ((3.0 / 2.0) * r);
    return { x, y };
}

export default function PathfindingPage() {
    const [rooms, setRooms] = useState([]);
    const [path, setPath] = useState([]);
    const [from, setFrom] = useState(null);
    const [to, setTo] = useState(null);
    const [loading, setLoading] = useState(false);
    const selectedRoom = rooms[0];

    useEffect(() => {
        (async () => {
            await promisified({
                cmd: "getWorld",
            })
                .then((r) => setRooms(r))
                .catch(console.error);
        })();
    }, [setRooms]);

    useEffect(() => {
        if (from && to && selectedRoom) {
            setLoading(true);
            (async () => {
                await promisified({
                    cmd: "findPath",
                    from: {
                        room: selectedRoom.room_id,
                        roomPos: from,
                    },
                    to: {
                        room: selectedRoom.room_id,
                        roomPos: to,
                    },
                })
                    .then((p) => setPath(p))
                    .catch(console.error);
                setLoading(false);
            })();
        }
    }, [setLoading, selectedRoom, from, to, setPath]);

    return (
        <>
            <Header />
            <div
                style={{
                    width: "80%",
                }}
            >
                {!selectedRoom ? (
                    "No rooms :("
                ) : (
                    <main>
                        <svg viewBox={`0 -50 1300 1200`}>
                            {selectedRoom.terrain.map(([[q, r], t], i) => {
                                const pos = axialToPixelPointy(10, { q, r });
                                return (
                                    <HexTile
                                        pos={pos}
                                        key={i}
                                        scale={10}
                                        tileTy={t}
                                        onHoverCb={() =>
                                            !loading && setTo([q, r])
                                        }
                                        onClickCb={() =>
                                            !loading && setFrom([q, r])
                                        }
                                    />
                                );
                            })}
                            {path
                                ? path.map(({ roomPos: [q, r] }, i) => (
                                      <HexTile
                                          pos={axialToPixelPointy(10, {
                                              q,
                                              r,
                                          })}
                                          key={i}
                                          scale={9}
                                          onHoverCb={() => {}}
                                          onClickCb={() =>
                                              !loading && setFrom([q, r])
                                          }
                                          color="blue"
                                      />
                                  ))
                                : null}
                        </svg>
                    </main>
                )}
            </div>
        </>
    );
}

function HexTile({ pos, scale, tileTy, onHoverCb, onClickCb, color }) {
    if (!color) color = "red";
    switch (tileTy) {
        case "bridge":
            color = "green";
            break;
        case "plain":
            color = "yellow";
            break;
    }

    const width = scale * SQRT_3;
    const height = scale * 2;

    const vertices = [
        [width / 2, height / 4],
        [width, 0],
        [width, -height / 2],
        [width / 2, (-height * 3) / 4],
        [0, -height / 2],
    ].map(([x, y]) => [x + pos.x, y + pos.y]);

    let path = `M ${pos.x} ${pos.y}`;
    for (let pos of vertices) {
        path = ` ${path} L ${pos[0]} ${pos[1]}`;
    }

    return (
        <path
            d={path}
            fill={color}
            onClick={onClickCb}
            onMouseOver={onHoverCb}
        />
    );
}
