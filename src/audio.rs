use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub fn play() -> Result<(), Box<dyn std::error::Error>> {
    let test_file = "test.wav";
    let reader = Box::leak(Box::new(hound::WavReader::open(test_file).unwrap())); 

    let host = cpal::default_host();

    let device = host.default_output_device().unwrap();
    println!("Output device: {}", device.name()?);

    let config = device.default_output_config().unwrap();
    println!("Default output config: {:?}", config);
    println!("config {:?}", config.sample_format());
 
    let supported_config = config.clone();
    let config: cpal::StreamConfig = config.into();

    let sample_rate = config.sample_rate.0 as f32;
    println!("Device sample rate {}", sample_rate);
    let channels = config.channels as usize;
    let (min, max) = match supported_config.buffer_size() {
        cpal::SupportedBufferSize::Range { min, max } => { (*min as f32, *max as f32) },
        cpal::SupportedBufferSize::Unknown => { (0.0, f32::MAX) }
    };
    let range = max - min;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let spec = reader.spec();
    let mut samples = reader.samples::<i16>();
    // This is sterio format so divide by 2 for mono
    let play_time = samples.size_hint().0 as f64 / spec.sample_rate as f64 / 2.0;
    println!("Playtime {}", play_time);
    let mut next_value = move || -> i16 {
        //sample_clock = (sample_clock + 1.0) % sample_rate;
        //(sample_clock * 160.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
        let _ = samples.next();
        let val = samples.next().unwrap_or(Ok(0)).unwrap();
        //println!("raw {}", val);
        //let a = (val - min) / range;
        //println!("transformed {}", a);
        val / 2
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            for frame in data.chunks_mut(channels) {
                let value = next_value();
                for sample in frame.iter_mut() {
                    *sample = value;
                }
            }
        },
        err_fn,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_secs_f64(play_time));

    Ok(())
}
