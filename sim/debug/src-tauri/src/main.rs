#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod cmd;

use cmd::generate_world;

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
                        // definitions for your custom commands from Cmd here
                        Cmd::GenerateWorld {
                            room_radius,
                            world_radius,
                            callback,
                            error,
                        } => tauri::execute_promise(
                            _webview,
                            move || {
                                let w = generate_world(world_radius, room_radius);
                                Ok(w)
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
