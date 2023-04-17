// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

use player::{mpv::MpvPlayer, PlayerEvent};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::{ops::DerefMut, sync::Mutex};
use tauri::{Manager, State};

mod player;

#[tauri::command]
fn play(url: String, player: State<AppState>) {
    let tx = player.lock().unwrap().tx.clone();
    if let Some(p) = player.lock().unwrap().player.as_ref() {
        if let Err(err) = p.stop() {
            println!("Error stopping player: {:?}", err);  
        }
    }
    player.lock().unwrap().player = None;
    let p = MpvPlayer::new(tx);
    p.play(&url).expect("Failed to play");
    player.lock().unwrap().player = Some(p);
}

#[tauri::command]
fn stop(player: State<AppState>) {
    if let Some(p) = player.lock().unwrap().player.as_ref() {
        println!("Stopping player");
        if let Err(err) = p.stop() {
            println!("Error stopping player: {:?} ;", err);
        }
    }

    player.lock().unwrap().player = None;
}

struct App {
    player: Option<MpvPlayer>,
    tx: Sender<PlayerEvent>,
}

type AppState = Arc<Mutex<App>>;

impl App {
    pub fn new(tx: Sender<PlayerEvent>) -> Arc<Mutex<App>> {
        Arc::new(Mutex::new(App { player: None, tx }))
    }
}

fn main() {
    let (tx, rx) = std::sync::mpsc::channel::<crate::player::PlayerEvent>();
    let state = App::new(tx);
    tauri::Builder::default()
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![play, stop])
        .setup(|app| {
            // ...

            let app_handle = app.handle();
            tauri::async_runtime::spawn(async move {
                // A loop that takes output from the async process and sends it
                // to the webview via a Tauri Event
                loop {
                    if let Ok(player_event) = rx.recv() {
                        match player_event {
                            PlayerEvent::Progress(progress) => {
                                app_handle.emit_all("progress", Some(progress)).unwrap();
                            }
                            PlayerEvent::Pause(pause) => {
                                app_handle.emit_all("pause", Some(pause)).unwrap();
                            }
                            PlayerEvent::Duration(duration) => {
                                app_handle.emit_all("duration", Some(duration)).unwrap();
                            }
                            PlayerEvent::Exit => {
                                app_handle.emit_all::<Option<()>>("exit", None).unwrap();
                            }
                        }
                        //println!("player_event: {:?}", player_event);
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
