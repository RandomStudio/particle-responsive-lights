# Particle Lights Simulator

Each chime is a "particle" in this system.

## Development setup
Paho Eclipse MQTT is actually a C library, so it has some dependencies. On Mac, `brew install openssh` and `brew install cmake` if necessary.

- Clone the repo
- Run `cargo run`

That's it. (Assuming your Rust environment is reasonably up to date.)

## Tech stack
- [Nannou](https://nannou.cc/): creative coding framework
- [Tween crate](https://docs.rs/tween/2.0.0/tween/index.html) as per the OG Robert Penning ease functions ([demo](https://easings.net/#))
- [Strum](https://crates.io/crates/strum) and [Strum macros](https://crates.io/crates/strum_macros), specfically for dealing with enums in a way that is convenient for the GUI
- [artnet_protocol](https://docs.rs/artnet_protocol/0.4.1/artnet_protocol/index.html)) for ArtNet/DMX output
- MQTT + MessagePack = Tether

___
## Remote Triggers
### Trigger a single fixture

Example:
```
tether-send --host localhost --topic dummy/dummy/lightTriggers --message=\{\"id\":7\,\"targetBrightness\":0.7\,\"attackDuration\":1000,\"finalBrightness\":0.2,\"transmissionRange\":0\}
```
Other than `id` and `targetBrightness`, all fields are optional. So the following also works:
```
tether-send --host localhost --topic dummy/dummy/lightTriggers --message=\{\"id\":0\,\"targetBrightness\":1.0\}
```

### Fade all lights simultaneously
Example - all on to full brightness:
```
tether-send --host localhost --topic dummy/dummy/lightReset --message=\{\"targetBrightness\":1.0\}
```

To black:
```
tether-send --host localhost --topic dummy/dummy/lightReset --message=\{\"targetBrightness\":0\}
```

Optional duration - if not provided, the fade will be "instant". Here's an example with a 3 second duration, to half-brightness:
```
tether-send --host localhost --topic dummy/dummy/lightReset --message=\{\"targetBrightness\":0.5\,\"fadeDuration\":3000\}
```
