# Tauri icons

Sourced from `docs/handoff/vaner-desktop/brand/png/` in the Vaner
brand package.

| File                 | Source             | Size     | Use                              |
|----------------------|--------------------|----------|----------------------------------|
| `32x32.png`          | `mark-light-32`    | 32×32    | Tauri default / window icon      |
| `128x128.png`        | `mark-light-128`   | 128×128  | Bundler (128px slot)             |
| `128x128@2x.png`     | `mark-light-256`   | 256×256  | Bundler HiDPI                    |
| `icon.png`           | `mark-light-512`   | 512×512  | `generate_context!()` default    |
| `tray.png`           | `mark-light-48`    | 48×48    | System-tray icon (colored mark)  |
| `tray-mono.png`      | `mark-white-24`    | 24×24    | Alt tray for pure-mono themes    |

All PNGs are RGBA with transparent backgrounds. The colored `tray.png`
matches the SwiftUI macOS app's menu-bar icon — Vaner keeps its
amber satellite on the panel.

To refresh from the brand package:

```bash
BRAND="/path/to/handoff/vaner-desktop/brand/png"
DEST="src-tauri/icons"
cp "$BRAND/mark-light-32.png"  "$DEST/32x32.png"
cp "$BRAND/mark-light-128.png" "$DEST/128x128.png"
cp "$BRAND/mark-light-256.png" "$DEST/128x128@2x.png"
cp "$BRAND/mark-light-512.png" "$DEST/icon.png"
cp "$BRAND/mark-light-48.png"  "$DEST/tray.png"
cp "$BRAND/mark-white-24.png"  "$DEST/tray-mono.png"
```

Brand rules (from the handoff README):
- Never recolor outside the provided mark variants.
- Minimum display size: 16px for the mark alone.
- Purple/amber pairing is the brand — don't substitute.
