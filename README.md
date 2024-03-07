`extol_pixel_font` allows rendering fonts that are stored as a single image (typically PNG), with each letter at a given location. This is common in game development, especially for pixel art fonts, since it allows the use of colors and creating a font can be done using any image editor as opposed to specialized software.

## Notes

For pixel-sharp text rendering:

- Locating the pixel font text at integer coordinates may cause it to be blurry. I'm not sure why.
