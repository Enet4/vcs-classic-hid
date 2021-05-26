## audio-monitor

An audio monitor application!
Watch your joystick LEDs glow to the sound of your computer!

This application uses `cpal` and spectrum analysis
to listen to audio emitted by the default output device
and process it into LED intensities.

This has been tested to work on Linux with ALSA as the sound driver,
but it might also work on other platforms with a bit of tweaking.

### Running

In the project's root folder:

```sh
cargo run --release --bin vcs-classic-hid-cat-mouse
```

Press the Fuji button at any time to end the program.
It will also close itself after a while.

