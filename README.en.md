# wiu

[Français](README.md) · **English** · [简体中文](README.zh-CN.md) · [Español](README.es.md) · [Русский](README.ru.md)

A Windows x64 image viewer written in Rust. It opens photos without editing or modifying them.

## Usage

When building from source, first compile the application and generate the `dist/` folder as described in [Building](#building). The `target/` and `dist/` folders are generated locally and are not tracked by Git.

Open `dist/wiu.exe`. The application starts in a maximized window with the Windows title bar and a dialog for selecting one or more images. Canceling leaves the window ready to open an image. Drag and drop is also supported. Press F11 to enter full screen, and F11 or Esc to leave it.

```powershell
.\dist\wiu.exe "C:\Photos\summer 2026.jpg"
.\dist\wiu.exe "C:\Photos\a.png" "C:\Photos\b.tiff"
```

| Shortcut | Action |
| --- | --- |
| Ctrl+O | Choose images |
| Left/right arrows | Previous/next image |
| Mouse wheel | Zoom around the cursor |
| Drag with the left mouse button | Pan the image |
| 0 / Adapter | Fit to window |
| 1 / 100 % | Actual size |
| F11 / Esc | Enter/leave full screen |
| Space | Pause/resume GIF playback |

Opening a single image lets you browse compatible images in the same folder, sorted by name. Passing multiple paths limits browsing to that selection. Display respects EXIF orientation for formats decoded by `image`. Opening a file never changes its bytes.

## Windows Explorer integration

Click **Installer** in the application window, or run:

```powershell
.\dist\wiu.exe --install
```

This copies wiu to `%LOCALAPPDATA%\Programs\wiu\wiu.exe` and registers the application under **Open with**, along with the **Ouvrir avec wiu** context menu action, for the current user. The **Installer** button disappears after a successful installation. If you launch another executable whose contents differ from the installed copy, a **Mettre à jour** button appears. It disappears after updating; identical copies show no installation button. Close other wiu windows before updating. On Windows 11, the classic action may appear under **Show more options**. No administrator rights are required. The user still chooses the default application for double-clicking images in Windows.

To remove these entries, run `wiu.exe --uninstall`. Then close wiu and delete its installation folder if desired. Installation can be repeated; to replace an installed executable, close that copy first and install from the new portable version.

## Formats and limits

- JPEG/JPG/JFIF, PNG, BMP, TIFF, ICO, PNM/PPM/PGM/PBM and TGA: built-in Rust decoders.
- GIF: built-in Windows decoder, with animation transparency, frame delays and frame disposal handling.
- WebP and AVIF: first frame only, using WIC codecs available on Windows. Windows WebP/AV1 image extensions may be required. The application reports a clear error if a codec is missing. Support is therefore not guaranteed on Windows installations without these codecs.
- SVG: native Direct2D rendering on Windows 10/11, as a static image at the document's dimensions. Some complex SVG features (filters, scripts, animations) are not supported by Direct2D. External references are rejected.
- Multipage TIFF: first page only. ICO: representation selected by the decoder. No HEIC or RAW support, or advanced color profile/HDR management.
- Maximum file size: 128 MiB. Decoded pixels, or all frames of a GIF combined, are limited to 256 MiB. Loading runs sequentially outside the user interface; outdated requests are ignored.

## Building

Use stable Rust (at least 1.92 for the UI), with the `x86_64-pc-windows-gnu` target and MinGW (`gcc`, `windres`) on your PATH.

```powershell
cargo build --release --locked
cargo test --locked
```

Dependency versions are locked in `Cargo.lock`. You can add `--offline` to Cargo commands if all dependencies are already cached. The UI uses eframe/egui; dialogs, GIF decoding and optional codecs use Windows; SVG uses Direct2D software rendering. Nothing is downloaded at runtime.

The executable is generated at `target/release/wiu.exe`. After building, use Python to generate the distribution folder:

```powershell
python tools/package.py
```

The script copies the executable, all five versions of this documentation, the project license and dependency licenses into `dist/`. It uses `cargo metadata --offline`, so dependencies must already be cached.

To check decoding without opening a window, run `wiu.exe --check "image.png" "image.svg"` (exit code 0 if all files can be read, 1 otherwise). Tests create their samples in `target/fixtures`.

Registry changes are limited to `HKCU\Software\Classes\Applications\wiu.exe`, `HKCU\Software\Classes\wiu.Image`, the `wiu` verbs under `SystemFileAssociations`, `HKCU\Software\wiu`, and the `wiu` value in `RegisteredApplications`.

## License

wiu is distributed under the [MIT license](LICENSE). Dependency licenses are included in the generated distribution folder.
