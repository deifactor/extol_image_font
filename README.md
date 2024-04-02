`extol_pixel_font` allows rendering fonts that are stored as a single image (typically PNG), with each letter at a given location. This is common in game development, especially for pixel art fonts, since it allows the use of colors and creating a font can be done using any image editor as opposed to specialized software.

## How to use

Note that for pixel-accurate rendering:

- Locating the text at integer coordinates can cause it to be blurry. I'm not sure why. You'll want to offset the x/y coordinates by a small amount like 0.1.
- You need to disable anti-aliasing via `app.insert_resource(Msaa::Off)`. Otherwise, you'll get fragments of other glyphs in your text. extol_pixel_font does this for you.
- Any image you use with this needs to use a nearest-neighbor sampler. Using `DefaultPlugins.build().set(ImagePlugin::default_nearest())` will set this as the default for all your images; if you don't want that, grab the `Image` and do `image.sampler = ImageSampler::nearest()`.

See [the basic example] for sample usage using the `bevy_asset_loader` crate to construct handles to the texture layout and image. 

[the basic example]: https://github.com/deifactor/extol_pixel_font/blob/main/examples/basic.rs
