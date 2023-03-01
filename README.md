# Particle Lights Simulator

Each chime is a "particle" in this system.

## Development setup
Generally, it should be as simple as:
- Clone the repo
- Run `cargo run`

However, Paho Eclipse MQTT is actually a C library, so it has some dependencies of its own. If you find that the build fails, you might need to `brew install openssh` and `brew install cmake` (on MacOS).

If you're testing without an ArtNet device available, run with `--artnet.broadcast`. You can monitor output, if you like, with a tool such as [ArtNetView](https://artnetview.com/).

## Tech stack
- [Nannou](https://nannou.cc/): creative coding framework
- [Tween crate](https://docs.rs/tween/2.0.0/tween/index.html) as per the OG Robert Penning ease functions ([demo](https://easings.net/#))
- [Strum](https://crates.io/crates/strum) and [Strum macros](https://crates.io/crates/strum_macros), specfically for dealing with enums in a way that is convenient for the GUI
- [artnet_protocol](https://docs.rs/artnet_protocol/0.4.1/artnet_protocol/index.html)) for ArtNet/DMX output
- MQTT + MessagePack = Tether

___
## Saving/loading settings
By default, your custom settings will be saved to a file named `settings.json`, and these are what will be loaded on startup.

Keep in mind that these settings apply when you are clicking on the light fixtures to test out various effects, and they will be used as defaults in "live" remote-controlled animations as well, **but some aspects of specific animation effects can be overridden by incoming trigger messages**. 

For example, you may specify a (long) release duration of 9000ms, and this value will indeed be used if the LightTriggerMessage does not specify anything, e.g.:
```
{ id: 0, targetBrightness: 0.9 }
```
...but will be overridden to 1000ms if the incoming message specifies it:
```
{ id: 0, targetBrightness: 0.9, releaseDuration: 1000 }
```


## Command-line arguments
Pass `--help` to see the full list, e.g. `cargo run -- --help`

If testing locally, you may want to use ArtNet Broadcast mode and disable Tether, i.e.
`--artnet.broadcast --tether.disable`

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
