# Cosmic Live Wallpaper
animated live wallpaper built in Rust using the [Bevy](https://bevyengine.org/) game engine.

## Features

- **Dynamic Color Syncing**: Analyzes an inputted wallpaper image on startup, extracts an 8-color scheme, and dynamically recolors the SVG elements to perfectly match your desktop.
- **Resolution Independence**: Automatically scales the SVG graphics to fit your screen perfectly, no matter your monitor's aspect ratio or resolution.
- **Low Overhead**: Runs as a lightweight Wayland surface via `bevy_live_wallpaper`, stripping out unnecessary Bevy features (like 3D rendering and audio) to keep performance high and binary size small.

## Requirements

- Linux with Wayland
- Nix (for development and building)

## Getting Started

This project is packaged with a Nix flake to provide a fully reproducible development and build environment.

### Running Locally

To enter the development shell and run the wallpaper:

```bash
# Enter the nix shell
nix develop

# Run the project (optionally pass a wallpaper image for color syncing)
cargo run -- --wallpaper /path/to/your/wallpaper.jpg
```


## Credits

Special thanks to the creator of the [Caelestia Material Wave Wallpaper](https://www.figma.com/design/SI9495HhPhuREO6vQjDj1h/Caelestia-Material-Wave-Wallpaper?node-id=0-1&t=2DURRhAC8cQnNHwk-1) Figma design, which served as the beautiful underlying SVG artwork for this live wallpaper.

## License
MIT
