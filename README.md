`extol_pixel_font` allows rendering fonts that are stored as a single image (typically PNG), with each letter at a given location. This is common in game development, especially for pixel art fonts, since it allows the use of colors and creating a font can be done using any image editor as opposed to specialized software.

## How to use

Note that for pixel-accurate rendering, locating the text at integer coordinates can cause it to be blurry. I'm not sure why. If this happens, you'll want to offset the x/y coordinates by a small amount like 0.1.

See [the bevy_ui example] for sample usage using the `bevy_asset_loader` crate to construct handles to the texture layout and image, or [the sprite example] if you want to use pixel fonts 'in the world' (such as for flying damage text).

[the basic example]: https://github.com/deifactor/extol_pixel_font/blob/main/examples/basic.rs
[the bevy_ui example]: https://github.com/deifactor/extol_pixel_font/blob/main/examples/bevy_ui.rs
