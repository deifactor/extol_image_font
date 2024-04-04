`extol_image_font` allows rendering fonts that are stored as a single image (typically PNG), with each letter at a given location. This is common in game development, especially for pixel art fonts, since it allows the use of colors and creating a font can be done using any image editor as opposed to specialized software. These are also sometimes known as 'pixel fonts', but I choose the name 'image font' to be more precise (since bitmap fonts stored in OTB could also be called 'pixel fonts').

## Features

**Supported**

- Unicode (anything that fits in a single codepoint)
- Specifying the coordinates with a string containing the letters in proper order (see the example asset)
- Manually specifying the rects (including non-uniform sizes)

**Future work**

- Padding and offsets for automatic texture layout
- Newlines embedding in strings

**Out of scope**

- Rendering from 'actual' bitmap fonts
- Automatic line wrapping

### Caveats

- You need to have a portion of the texture that's just blank and 'map' the space character to it.
- Newlines are not currently supported.

## How to use

**Note that for pixel-accurate rendering, locating the text at integer coordinates 'in the world' can cause it to be blurry.** I'm not sure why. If this happens, you'll want to offset the x/y coordinates by a small amount like 0.1. This seems to be less of an issue using this crate with bevy_ui.

Just take any entity with a `Handle<Image>` or `UiImage` component, such as something created with a `SpriteBundle` or `ImageBundle`, and add a `ImageFontText` component to it.

See [the bevy_ui example] for sample usage using the `bevy_asset_loader` crate to construct handles to the texture layout and image, or [the sprite example] if you want to use pixel fonts 'in the world' (such as for flying damage text).

[the sprite example]: https://github.com/deifactor/extol_image_font/blob/main/examples/sprite.rs
[the bevy_ui example]: https://github.com/deifactor/extol_image_font/blob/main/examples/bevy_ui.rs

If you're not using `bevy_ui`, you can disable the `bevy_ui` feature (enabled by default) to avoid taking a dependency on that.

This crate uses the `image` crate to load images, but only enables PNG support by default. If you need some other format, add your own dependency on (the same version of) `image` and enable the relevant features.

## How it works


## Credits

The sample font is by [gnsh](https://opengameart.org/content/bitmap-font-0).
