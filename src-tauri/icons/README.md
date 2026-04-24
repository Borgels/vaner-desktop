# Tauri icons

Drop the following files here before running `pnpm tauri build`:

| File         | Size     | Notes                                   |
|--------------|----------|-----------------------------------------|
| `32x32.png`  | 32×32    | Tray icon — should be template-style    |
| `128x128.png`| 128×128  | App icon (bundled)                      |
| `icon.png`   | 512×512  | High-DPI app icon                       |
| `icon.ico`   | multi    | Future Windows build                    |

The tray icon is used on the Linux/Windows/macOS tray. On Linux,
template-style (monochrome, transparent background) renders cleanly
across GNOME/KDE themes.

For v1 CI builds without icons, `tauri.conf.json > bundle.active` is
set to `false`; `pnpm tauri dev` runs without them.

## Generating the base set

The Vaner brand mark lives in
`docs/handoff/vaner-desktop/brand/svg/` in the main Vaner repo.
Convert with ImageMagick:

```bash
magick convert vmark.svg -resize 32x32 32x32.png
magick convert vmark.svg -resize 128x128 128x128.png
magick convert vmark.svg -resize 512x512 icon.png
```

Or use [tauri-icon](https://tauri.app/v2/guides/features/icons/) to
generate the whole set from a single high-res source.
