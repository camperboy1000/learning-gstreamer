use gst::prelude::*;
use gstreamer as gst;

struct Stream {
    pipeline: gst::Pipeline,
    source: gst::Element,
    audioconvert: gst::Element,
    audioresample: gst::Element,
    audiosink: gst::Element,
    videoconvert: gst::Element,
    videosink: gst::Element,
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
        audioconvert: gst::ElementFactory::make("audioconvert")
            .name("audioconvert")
            .build()
            .expect("Failed to create audioconvert element"),
        audioresample: gst::ElementFactory::make("audioresample")
            .name("audioresample")
            .build()
            .expect("Failed to create audioresample element"),
        audiosink: gst::ElementFactory::make("autoaudiosink")
            .name("audiosink")
            .build()
            .expect("Failed to create autoaudiosink element"),
        videoconvert: gst::ElementFactory::make("videoconvert")
            .name("videoconvert")
            .build()
            .expect("Failed to create videoconvert"),
        videosink: gst::ElementFactory::make("autovideosink")
            .name("videosink")
            .build()
            .expect("Failed to create autovideosink element"),
    };

    // Add and link elements to pipeline
    // NOTICE: Source element has not been connected yet
    stream
        .pipeline
        .add_many(&[
            &stream.source,
            &stream.audioconvert,
            &stream.audioresample,
            &stream.audiosink,
            &stream.videoconvert,
            &stream.videosink,
        ])
        .expect("Failed to add elements to pipeline");

    gst::Element::link_many(&[
        &stream.audioconvert,
        &stream.audioresample,
        &stream.audiosink,
    ])
    .expect("Failed to link audio elements");

    stream
        .videoconvert
        .link(&stream.videosink)
        .expect("Failed to link video elements");

    stream.source.connect_pad_added(move |src, src_pad| {
        println!("Received new pad {} from {}", src_pad.name(), src.name());

        let audio_sink_pad = stream
            .audioconvert
            .static_pad("sink")
            .expect("Failed to get static sink pad from audioconvert");
        let video_sink_pad = stream
            .videoconvert
            .static_pad("sink")
            .expect("Failed to get sink pad from videoconvert");

        if audio_sink_pad.is_linked() && video_sink_pad.is_linked() {
            println!("We are already linked. Ignoring...");
            return;
        }

        let new_pad_caps = src_pad
            .current_caps()
            .expect("Failed to get caps of new pad");
        let new_pad_struct = new_pad_caps
            .structure(0)
            .expect("Failed to get first structure of pad");
        let new_pad_type = new_pad_struct.name().as_str();

        match new_pad_struct.name().as_str() {
            "audio/x-raw" => {
                if audio_sink_pad.is_linked() {
                    println!("Audio pad is already linked. Ignoring...");
                    return;
                };

                match src_pad.link(&audio_sink_pad) {
                    Ok(_) => println!("Link succeeded (type {})", new_pad_type),
                    Err(_) => println!("Type is {} but link failed", new_pad_type),
                };
            }
            "video/x-raw" => {
                if video_sink_pad.is_linked() {
                    println!("Video pad is already linked. Ignoring...");
                    return;
                }

                match src_pad.link(&video_sink_pad) {
                    Ok(_) => println!("Link succeeded (type {})", new_pad_type),
                    Err(_) => println!("Type is {} but link failed", new_pad_type),
                }
            }
            _ => {
                println!(
                    "Pad had type {} which is not raw audio or video. Ignoring...",
                    new_pad_type
                )
            }
        }
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
