# Winget manifests for Vaner Desktop

`winget install Vaner.Desktop` reads the [winget-pkgs](https://github.com/microsoft/winget-pkgs) registry. To get Vaner Desktop into that registry we open a PR with the manifest triple here.

## Layout

The directory layout under `winget/` mirrors the path the manifests need to land at inside `microsoft/winget-pkgs`:

```
manifests/v/Vaner/Desktop/<version>/
  Vaner.Desktop.yaml                 # version manifest
  Vaner.Desktop.installer.yaml       # installer manifest (URL + sha256)
  Vaner.Desktop.locale.en-US.yaml    # default-locale (publisher / description)
```

Schema reference: [Microsoft winget-pkgs manifest schemas, v1.6.0](https://github.com/microsoft/winget-pkgs/tree/master/doc/manifest).

## Releasing a new version

Each tagged Vaner Desktop release ships a `vaner-desktop_<ver>_x64-setup.exe` (Tauri NSIS bundle).

1. Compute the SHA256 of the published `.exe`:
   ```bash
   curl -sL -o /tmp/vd.exe \
     https://github.com/Borgels/vaner-desktop/releases/download/v<NEW>/vaner-desktop_<NEW>_x64-setup.exe
   sha256sum /tmp/vd.exe
   ```
2. Copy the `0.2.4/` directory to a new `<NEW>/` directory and replace the version + URL + SHA256 in all three files.
3. Validate locally with `winget validate <NEW>/` (or `wingetcreate validate <NEW>/`) if you have winget on Windows.
4. Open a PR against [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) by copying the `manifests/v/Vaner/Desktop/<NEW>/` directory into that repo's tree at the same path. Microsoft's bots run automated checks; the PR gets reviewed and merged within a day or two on a green run.

## Why this lives in the desktop repo

The manifests reference exact GitHub-release artifact URLs + their SHA256 — every Vaner Desktop release needs a corresponding manifest bump. Keeping the source-of-truth files next to the release workflow (rather than in a separate winget-only repo) means a release engineer touches one repo per version bump, not two.

## Automation hook (future)

Once 0.2.4 is accepted by `microsoft/winget-pkgs`, we can wire `wingetcreate update Vaner.Desktop --urls <new-exe-url> --version <new-version> --submit` into `release.yml` so subsequent releases auto-PR the manifest. For 0.2.4 we do this manually to confirm Microsoft's bot accepts the package identifier.
