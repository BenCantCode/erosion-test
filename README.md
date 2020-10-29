# Erosion

The following is my implementation of a "terrain gradient" calculator, which can be used to determine the steepness and the slope angle of any point on a heightmap.
This demonstration will output 3 images:

1. heightmap.bmp - the raw generated terrain from an fBm function to use as example terrain. This is generated for demonstration purposes, and this algorithm can, in theory, be applied to any image.
2. angle.bmp - the angle of the slope, with the angle represented as hue (because hue is circular).
3. steep.bmp - a map of how steep each pixel is.

This was designed in such a way to be used for particle simulations rather
than generating images, which is why I'm doing subpixel interpolation.
Regardless, it looks pretty cool when graphed as an image too.

## Installation

1. Install Rust via https://rustup.rs
2. Navigate to this folder and use the command ```cargo run```. The three files above will be generated.

## Source

Commented source is available at src/main.rs