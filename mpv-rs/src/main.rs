use libmpv::{
    events::{Event, PropertyData},
    Error, FileState, Format, Mpv,
};

fn main() -> Result<(), Error> {
    let url = "https://jelly.maulik.tech/Items/a1a5f212d7ee32793e12b32efc059f4e/Download?api_key=0f7a3288866b466fba0465df0c4d9ac4";
    println!("Hello, world!");
    let mpv = Mpv::new()?;

    mpv.set_property("volume", 15)?;
    mpv.set_property("osc", "")?;
    mpv.set_property("input-default-bindings", true)?;
    let mut ev_ctx = mpv.create_event_context();
    ev_ctx.disable_deprecated_events()?;
    ev_ctx.observe_property("volume", Format::Int64, 0)?;
    ev_ctx.observe_property("demuxer-cache-state", Format::Node, 0)?;
    ev_ctx.observe_property("pause", Format::Flag, 0)?;
    ev_ctx.observe_property("time-pos", Format::Int64, 0)?;
    mpv.playlist_load_files(&[(&url, FileState::AppendPlay, None)])?;

    loop {
        let ev = ev_ctx.wait_event(600.).unwrap_or(Err(Error::Null));

        match ev {
            Ok(Event::EndFile(r)) => {
                println!("Exiting! Reason: {:?}", r);
                break;
            }

            Ok(Event::PropertyChange {
                name: "demuxer-cache-state",
                change: PropertyData::Node(_mpv_node),
                ..
            }) => {
                // println!("Seekable ranges updated: {:?}", mpv_node);
            }
            Ok(e) => println!("Event triggered: {:?}", e),
            Err(e) => println!("Event errored: {:?}", e),
        }
    }

    Ok(())
}
