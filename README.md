# Particle Lights Simulator

Each chime is a "particle" in this system.

## Development setup
- Clone the repo
- Run `cargo run`

That's it. (Assuming your Rust environment is reasonably up to date.)

## Tech stack
- [Nannou](https://nannou.cc/): creative coding framework
- [Tween crate](https://docs.rs/tween/2.0.0/tween/index.html) as per the OG Robert Penning ease functions ([demo](https://easings.net/#))
- [Strum](https://crates.io/crates/strum) and [Strum macros](https://crates.io/crates/strum_macros), specfically for dealing with enums in a way that is convenient for the GUI
- [artnet_protocol](https://docs.rs/artnet_protocol/0.4.1/artnet_protocol/index.html)) for ArtNet/DMX output
- MQTT + MessagePack = Tether
