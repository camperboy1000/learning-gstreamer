use gst::prelude::*;

fn main() {
    // Initalize gstreamer
    gst::init().unwrap();

    // Create source and sink elements
    let source = gst::ElementFactory::make("videotestsrc")
        .name("source")
        .property_from_str("pattern", "smpte")
        .build()
        .expect("Failed to create source element");
    let sink = gst::ElementFactory::make("autovideosink")
        .name("sink")
        .build()
        .expect("Failed to create sink element");

    // Create empty pipeline
    let pipeline = gst::Pipeline::builder().name("test-pipeline").build();

    // Build pipeline
    pipeline.add_many(&[&source, &sink]).unwrap();
    source.link(&sink).expect("Failed to link source with sink");

    // Start playing
    pipeline
        .set_state(gst::State::Playing)
        .expect("Failed starting pipeline");

    // Wait till EOS or error is received
    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(_) => break,
            MessageView::Error(err) => {
                eprintln!(
                    "Error received from element {:?}: {}",
                    err.src().map(|s| s.path_string()),
                    err.error()
                );
                break;
            }
            _ => (),
        }
    }

    // Shutdown pipeline
    pipeline
        .set_state(gst::State::Null)
        .expect("Failed to shutdown pipeline");
}
