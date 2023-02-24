use gst::prelude::*;

struct Stream {
    pipeline: gst::Pipeline,
    source: gst::Element,
    convert: gst::Element,
    resample: gst::Element,
    sink: gst::Element,
}

fn main() {
    gst::init().unwrap();

    let uri =
        "https://www.freedesktop.org/software/gstreamer-sdk/data/media/sintel_trailer-480p.webm";

    let stream = Stream {
        pipeline: gst::Pipeline::builder().build(),
        source: gst::ElementFactory::make("uridecodebin")
            .name("source")
            .property_from_str("uri", uri)
            .build()
            .expect("Failed to create uridecodebin element"),
        convert: gst::ElementFactory::make("audioconvert")
            .name("convert")
            .build()
            .expect("Failed to create audioconvert element"),
        resample: gst::ElementFactory::make("audioresample")
            .name("resample")
            .build()
            .expect("Failed to create audioresample element"),
        sink: gst::ElementFactory::make("autoaudiosink")
            .name("sink")
            .build()
            .expect("Failed to create autoaudiosink element"),
    };

    // Add and link elements to pipeline
    // NOTICE: Source element has not been connected yet
    stream
        .pipeline
        .add_many(&[
            &stream.source,
            &stream.convert,
            &stream.resample,
            &stream.sink,
        ])
        .expect("Failed to add elements to pipeline");

    gst::Element::link_many(&[&stream.convert, &stream.resample, &stream.sink])
        .expect("Failed to link elements");

    stream.source.connect_pad_added(move |src, src_pad| {
        println!("Received new pad {} from {}", src_pad.name(), src.name());

        let sink_pad = stream
            .convert
            .static_pad("sink")
            .expect("Failed to get static sink pad from convert");
        if sink_pad.is_linked() {
            println!("We are already linked. Ignoring...");
            return;
        }

        let new_pad_caps = src_pad
            .current_caps()
            .expect("Failed to get caps of new pad");
        let new_pad_struct = new_pad_caps
            .structure(0)
            .expect("Failed to get first structure of pad");
        let new_pad_type = new_pad_struct.name();

        let is_audio = new_pad_type.starts_with("audio/x-raw");
        if !is_audio {
            println!(
                "Pad had type {} which is not raw audio. Ignoring...",
                new_pad_type
            );
            return;
        }

        match src_pad.link(&sink_pad) {
            Ok(_) => println!("Link succeeded (type {})", new_pad_type),
            Err(_) => println!("Type is {} but link failed", new_pad_type),
        };
    });

    // Start playing
    stream
        .pipeline
        .set_state(gst::State::Playing)
        .expect("Failed to start pipeline");

    let bus = stream.pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Error(err) => {
                eprintln!(
                    "Error received from element {:?}: {}",
                    err.src().map(|s| s.path_string()),
                    err.error()
                );
                eprintln!("Debug: {:?}", err.debug());
                break;
            }
            MessageView::StateChanged(state_changed) => println!(
                "Pipeline changed state from {:?} to {:?}",
                state_changed.old(),
                state_changed.current()
            ),
            MessageView::Eos(_) => break,
            _ => (),
        }
    }

    // Shutdown pipeline
    stream
        .pipeline
        .set_state(gst::State::Null)
        .expect("Faile to stop pipeline");
}
