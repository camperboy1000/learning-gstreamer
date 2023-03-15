use gst::prelude::*;
use gstreamer as gst;

struct CustomData {
    playbin: gst::Element,            // Our one and only element
    playing: bool,                    // Are we in the PLAYING state?
    terminate: bool,                  // Should we terminate execution?
    seek_enabled: bool,               // Is seeking enabled for this media?
    seek_done: bool,                  // Have we performed the seek already?
    duration: Option<gst::ClockTime>, // How long does this media last?
}

fn main() {
    gst::init().expect("Failed to initalize gstreamer");

    let uri =
        "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm";

    // Create the playbin element
    let playbin = gst::ElementFactory::make("playbin")
        .name("playbin")
        .property("uri", uri)
        .build()
        .expect("Failed to create playbin element");

    // Start the pipeline
    playbin
        .set_state(gst::State::Playing)
        .expect("Failed to start the pipeline");

    // Create custom data and listen to bus
    let bus = playbin.bus().expect("Failed to retreive the bus");
    let mut data = CustomData {
        playbin,
        playing: false,
        terminate: false,
        seek_enabled: false,
        seek_done: false,
        duration: gst::ClockTime::NONE,
    };

    while !data.terminate {
        match bus.timed_pop(100 * gst::ClockTime::MSECOND) {
            Some(msg) => handle_message(&mut data, &msg),
            None => {
                if data.playing {
                    // Get the current position
                    let position: gst::ClockTime = data
                        .playbin
                        .query_position()
                        .expect("Failed to get current position");

                    // Set the duration if we haven't set it yet
                    if data.duration == gst::ClockTime::NONE {
                        data.duration = data.playbin.query_duration();
                    }

                    println!("\rPosition {} / {}", position, data.duration.display());

                    // After 10s, skip to 30s only if seek is enabled and we haven't already seeked
                    if data.seek_enabled
                        && !data.seek_done
                        && position > 10 * gst::ClockTime::SECOND
                    {
                        // Perform the seek
                        println!("Reached 10 seconds, performing seek...");
                        data.playbin
                            .seek_simple(
                                gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                                30 * gst::ClockTime::SECOND,
                            )
                            .expect("Failed to seek");

                        // Prevent from seeking again
                        data.seek_done = true;
                    }
                }
            }
        }
    }

    data.playbin
        .set_state(gst::State::Null)
        .expect("Pipeline shutdown failed");
}

fn handle_message(data: &mut CustomData, msg: &gst::Message) {
    use gst::MessageView;

    match msg.view() {
        MessageView::Error(err) => {
            println!(
                "Error received from element {:?}: {} ({:?})",
                err.src().map(|s| s.path_string()),
                err.error(),
                err.debug()
            );
            data.terminate = true;
        }

        MessageView::Eos(_) => {
            println!("End of stream reached");
            data.terminate = true;
        }

        // Duration changed, update the duration value
        MessageView::DurationChanged(_) => data.duration = data.playbin.query_duration(),

        MessageView::StateChanged(state_changed) => {
            if state_changed
                .src()
                .map(|s| s == &data.playbin)
                .unwrap_or(false)
            {
                let new_state = state_changed.current();
                let old_state = state_changed.old();

                println!(
                    "Pipeline changed from state {:?} to {:?}",
                    old_state, new_state
                );

                data.playing = new_state == gst::State::Playing;
                if data.playing {
                    let mut seeking = gst::query::Seeking::new(gst::Format::Time);

                    if data.playbin.query(&mut seeking) {
                        let (seekable, start, end) = seeking.result();

                        data.seek_enabled = seekable;
                        match seekable {
                            true => println!("Seeking is ENABLED from {} to {}", start, end),
                            false => println!("Seeking is DISABLED"),
                        }
                    }
                }
            }
        }

        _ => (),
    }
}
