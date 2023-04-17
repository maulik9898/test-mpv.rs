use std::sync::{
    mpsc::{SendError, Sender},
    Arc, Mutex,
};

use crate::player::PlayerCommand;
use libmpv::{events::PropertyData, Error, FileState, Mpv};

use super::PlayerEvent;

pub struct MpvPlayer {
    tx: Arc<Mutex<std::sync::mpsc::Sender<PlayerCommand>>>,
}

impl MpvPlayer {
    pub fn new(tx_event: Sender<PlayerEvent>) -> Self {
        let mut mpv = Mpv::new().unwrap();
        mpv.set_property("volume", 15)
            .expect("Failed to set volume");
        mpv.set_property("osc", "").expect("Failed to set osc");
        mpv.set_property("input-default-bindings", "yes")
            .expect("Failed to set input-default-bindings");
        mpv.set_property("msg-level", "all=error")
            .expect("Failed to set msg-level");

        // mpv.set_property("input-cursor", true)
        //     .expect("Failed to set input-cursor");
        mpv.set_property("vo", "gpu").expect("Failed to set vo");
        mpv.set_property("border", "no")
            .expect("Failed to set no-border");
        //mpv.set_property("fullscreen", true).unwrap();
        mpv.set_property("input-vo-keyboard", "yes")
            .expect("Failed to set input-vo-keyboard");

        mpv.set_property("idle", "yes").expect("Failed to set idle");

        let (tx, rx) = std::sync::mpsc::channel::<crate::player::PlayerCommand>();

        std::thread::spawn(move || {
            // ev_ctx.observe_property("volume", Format::Int64, 0).unwrap();
            // ev_ctx.observe_property("pause", Format::Flag, 0).unwrap();
            // ev_ctx
            //     .observe_property("time-pos", Format::Int64, 0)
            //     .unwrap();

            let mut ctx = mpv.create_event_context();
            ctx.disable_deprecated_events()
                .expect("Failed to disable deprecated events");
            ctx.observe_property("time-pos", libmpv::Format::Int64, 0)
                .expect("Failed to observe time-pos");
            ctx.observe_property("pause", libmpv::Format::Flag, 0)
                .expect("Failed to observe pause");
            ctx.observe_property("duration", libmpv::Format::Int64, 0)
                .expect("Failed to observe duration");

            loop {
                let ev = ctx.wait_event(2.0).unwrap_or(Err(Error::Null));

                match ev {
                    Ok(libmpv::events::Event::EndFile(r)) => {
                        println!("Exiting! Reason: {:?}", r);
                        tx_event.send(PlayerEvent::Exit).unwrap();
                        break;
                    }
                    Ok(libmpv::events::Event::PropertyChange {
                        name: "time-pos",
                        change: PropertyData::Int64(pos),
                        ..
                    }) => {
                        tx_event.send(PlayerEvent::Progress(pos)).unwrap();
                    }

                    Ok(libmpv::events::Event::PropertyChange {
                        name: "pause",
                        change: PropertyData::Flag(pause),
                        ..
                    }) => {
                        tx_event.send(PlayerEvent::Pause(pause)).unwrap();
                    }

                    Ok(libmpv::events::Event::PropertyChange {
                        name: "duration",
                        change: PropertyData::Int64(duration),
                        ..
                    }) => {
                        tx_event.send(PlayerEvent::Duration(duration)).unwrap();
                    }
                    Ok(e) => println!("Event triggered: {:?}", e),
                    Err(Error::Null) => {}
                    Err(e) => println!("Error: {:?}", e),
                }

                if let Ok(event) = rx.try_recv() {
                    match event {
                        PlayerCommand::Play(path) => {
                            mpv.playlist_load_files(&[(&path, FileState::Replace, None)])
                                .expect(format!("Failed to load file: {}", path).as_str());
                        }
                        PlayerCommand::QueNext(path) => mpv
                            .playlist_load_files(&[(&path, FileState::AppendPlay, None)])
                            .expect(format!("Failed to load file: {}", path).as_str()),
                        PlayerCommand::Stop => {
                            mpv.command("quit", &[]).unwrap();
                        }
                    }
                }
            }

            println!("Exiting player thread");
        });

        Self {
            tx: Arc::new(Mutex::new(tx)),
        }
    }

    pub fn play(&self, path: &str) -> Result<(), SendError<PlayerCommand>> {
        self.dispatch(PlayerCommand::Play(path.to_string()))
    }

    fn dispatch(&self, event: PlayerCommand) -> Result<(), SendError<PlayerCommand>> {
        let tx = self.tx.lock().unwrap();
        tx.send(event.clone())
    }

    pub fn stop(&self) -> Result<(), SendError<PlayerCommand>> {
        self.dispatch(PlayerCommand::Stop)
    }
}
