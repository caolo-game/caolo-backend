#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod cmd;

use slog::{o, Drain};

use cmd::generate_world;

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_envlogger::new(drain).fuse();
    let drain = slog_async::Async::new(drain)
        .overflow_strategy(slog_async::OverflowStrategy::Block)
        .build()
        .fuse();
    let logger = slog::Logger::root(drain, o!());

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
                        } => {
                            let logger = logger.clone();
                            tauri::execute_promise(
                                _webview,
                                move || {
                                    let w = generate_world(logger, world_radius, room_radius);
                                    Ok(w)
                                },
                                callback,
                                error,
                            )
                        }
                        Cmd::MapNoise {
                            room,
                            room_radius,
                            seed,
                            callback,
                            error,
                        } => {
                            use cmd::generate_room_noise;

                            tauri::execute_promise(
                                _webview,
                                move || {
                                    let res = generate_room_noise(room, room_radius, seed);
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
