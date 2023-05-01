//! Monitors the default output device.
//!
//!

use std::sync::{Arc, Mutex};

use anyhow::{self, Context};
use clap::Parser;
use spectrum_analyzer::{self, FrequencyLimit, FrequencySpectrum, samples_fft_to_spectrum, windows::hann_window};
use vcs_classic_hid::{self, Device, LedReport, process_input};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Debug, Parser)]
struct App {
    /// The audio device to use
    #[clap(default_value = "default")]
    device: String,
    #[cfg(all(
        any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
        feature = "jack"
    ))]
    /// Use the JACK host
    #[clap(short = 'j', long = "--jack")]
    jack: bool,
}

fn main() -> Result<(), anyhow::Error> {
    let opt = App::parse();

    // Retrieve controller
    let joy = vcs_classic_hid::open().context("Could not open controller")?;

    let joy = Arc::new(Mutex::new(joy));

    // Conditionally compile with jack if the feature is specified.
    #[cfg(all(
        any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"),
        feature = "jack"
    ))]
    // Manually check for flags. Can be passed through cargo with -- e.g.
    // cargo run --release --example beep --features jack -- --jack
    let host = if opt.jack {
        cpal::host_from_id(cpal::available_hosts()
            .into_iter()
            .find(|id| *id == cpal::HostId::Jack)
            .expect(
                "make sure --features jack is specified. only works on OSes where jack is available",
            )).expect("jack host unavailable")
    } else {
        cpal::default_host()
    };

    #[cfg(any(
        not(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd")),
        not(feature = "jack")
    ))]
    let host = cpal::default_host();

    // Setup the input device and stream with the default input config.
    let device = if opt.device == "default" {
        host.default_output_device()
    } else {
        host.output_devices()?
            .find(|x| x.name().map(|y| y == opt.device).unwrap_or(false))
    }
    .expect("failed to find input device");

    println!("Listening to device: {}", device.name()?);

    let config = device
        .default_input_config()
        .expect("Failed to get default input config");
    println!("Default input config: {:?}", config);

    println!();

    let err_fn = move |err| {
        eprintln!("\nAn error occurred on stream: {}", err);
    };

    let joy1 = joy.clone();
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data, _: &_| handle_input_data_f32(data, joy1.clone()),
            err_fn,
            None,
        )?,
        other => {
            panic!("Unsupported sample format {:?}", other);
        }
    };

    stream.play()?;


    // enter game loop to keep listening to user input
    for _ in 0..5_000 {
        if let Some(input) = {
            let mut joy = joy.lock().unwrap();
            process_input(&mut *joy)?
        }{
            if input.button_fuji {
                break;
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    drop(stream);

    joy.lock().unwrap().reset_leds()?;
    std::thread::sleep(std::time::Duration::from_millis(50));

    Ok(())
}

fn handle_input_data_f32<D>(input: &[f32], joy: Arc<Mutex<D>>)
where
    D: Device,
{
    // apply hann window for smoothing; length must be a power of 2 for the FFT
    let hann_window = hann_window(&input[0..2048]);
    // calc spectrum
    let Ok(spectrum_hann_window) = samples_fft_to_spectrum(
        // (windowed) samples
        &hann_window,
        // sampling rate
        44100,
        // optional frequency limit: e.g. only interested in frequencies 50 <= f <= 150?
        FrequencyLimit::Range(20., 16_000.),
        // optional per element scaling function, e.g. `20 * log10(x)`; see doc comments
        None,
    ) else {
        return
    };

    let mut led = LedReport::new();

    let values: [u8; 24] = process_spectrum(&spectrum_hann_window);
    
    for (i, value) in values.iter().copied().enumerate() {
        led.set(i as u8, value);
    }

    joy.lock().unwrap().write(led).ok();
}

fn process_spectrum<const N: usize>(spectrum_hann_window: &FrequencySpectrum) -> [u8; N] {
    let mut out = [0; N];
    for (i, frs) in spectrum_hann_window.data().chunks(6).take(24).enumerate() {
        let mean_fr_val = frs.iter().map(|(_f, v)| v.val()).sum::<f32>() / (frs.len() as f32);
        out[i] = (mean_fr_val * 260.).round().min(255.) as u8;
    }

    out
}

