use gst::prelude::*;

fn main() {
    gst::init().unwrap();

    let uri =
        "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm";

    // Create a new video pipeine
    let pipeline = gst::parse_launch(&format!("playbin uri={}", uri)).unwrap();

    // Start playing the video
    pipeline.set_state(gst::State::Playing).unwrap();

    // Wait until EOS or error is received
    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                eprintln!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                break;
            }
            _ => (),
        }
    }

    // Shutdown sequence
    pipeline.set_state(gst::State::Null).unwrap();
}
