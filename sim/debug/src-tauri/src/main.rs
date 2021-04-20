#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod cmd;

use caolo_sim::prelude::*;
use cmd::find_path_world;
use std::pin::Pin;
use std::sync::Mutex;

pub fn create_world(world_radius: u32, room_radius: u32) -> std::pin::Pin<Box<World>> {
    let mut exc = SimpleExecutor;
    let world = exc
        .initialize(GameConfig {
            world_radius,
            room_radius,
            ..Default::default()
        })
        .unwrap();

    world
}

lazy_static::lazy_static! {
    static ref WORLD: Mutex<Pin<Box<World>>> = Mutex::new(create_world(8, 8));
}

fn main() {
    dotenv::dotenv().unwrap_or_default();
    tracing_subscriber::fmt::init();

    tauri::AppBuilder::new()
        .invoke_handler(move |_webview, arg| {
            use cmd::Cmd;

            match serde_json::from_str(arg) {
                Err(e) => Err(e.to_string()),
                Ok(command) => {
                    match command {
                        Cmd::FindPath {
                            from,
                            to,
                            callback,
                            error,
                        } => tauri::execute_promise(
                            _webview,
                            move || {
                                let mut world = WORLD.lock().unwrap();
                                let mut positions =
                                    world.unsafe_view::<WorldPosition, EntityComponent>();
                                if positions.table.at(from.room).is_none() {
                                    positions
                                        .table
                                        .insert(from.room, Default::default())
                                        .expect("Failed to init entites table");
                                }
                                let res = find_path_world(from, to, &*world);
                                Ok(res)
                            },
                            callback,
                            error,
                        ),
                        Cmd::GenerateWorld {
                            room_radius,
                            world_radius,
                            callback,
                            error,
                        } => tauri::execute_promise(
                            _webview,
                            move || {
                                let mut world = WORLD.lock().unwrap();
                                *world = create_world(world_radius, room_radius);
                                let mut exc = caolo_sim::prelude::SimpleExecutor;
                                exc.forward(&mut *world).unwrap(); // run system updates
                                let w = cmd::map_gen::render_terrain(&*world);
                                Ok(w)
                            },
                            callback,
                            error,
                        ),
                        Cmd::GetWorld { callback, error } => tauri::execute_promise(
                            _webview,
                            move || {
                                let world = WORLD.lock().unwrap();
                                let res = cmd::map_gen::get_terrain(&*world);
                                Ok(res)
                            },
                            callback,
                            error,
                        ),
                        Cmd::MapNoise {
                            room_radius,
                            seed,
                            callback,
                            error,
                        } => {
                            use cmd::generate_room_noise;

                            tauri::execute_promise(
                                _webview,
                                move || {
                                    let res = generate_room_noise(room_radius, seed);
                                    Ok(res)
                                },
                                callback,
                                error,
                            )
                        }
                    }
                    Ok(())
                }
            }
        })
        .build()
        .run();
}
