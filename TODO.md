# TODO: Telora

## Priority
- [ ] **Configurable Hotkeys**: Allow users to define their own shortcuts for toggle-type/toggle-copy.
- [ ] **Visual Feedback Improvements**: Add a volume meter or waveform to the OSD while recording.
- [ ] **Wayland Protocol Support**: Explore `wlr-virtual-keyboard-unstable-v1` for more robust typing on all Wayland compositors (currently uses a generic approach).

## Features
- [ ] **Continuous Dictation Mode**: A mode where the daemon transcribes in real-time without manual toggling.
- [ ] **Multi-language Auto-detection**: Leverage Whisper's language detection capabilities.
- [ ] **Architecture Refactor**: Move core logic from `telora-daemon/src/main.rs` to a `lib.rs` and implement a `Transcriber` trait for future engine support.

## UI/UX
- [ ] **Tray Icon**: Add a system tray icon for status monitoring and quick settings.
- [ ] **Configuration GUI**: A simple GTK window to edit `telora.toml`.
- [ ] **Integrated Model Manager**: A GUI for `telora-models` with download progress bars.
- [ ] **Model Detection UX**: Enhance the client (`telora`) to detect when the daemon fails due to a missing model and provide an interactive dialog to download it via `telora-models`.

## Maintenance
- [ ] **Unit Tests**: Increase coverage for audio processing and socket communication.
- [ ] **Integrity Checks**: Add SHA256 checksum verification for model downloads in `telora-models`.
- [ ] **CI/CD**: Automate binary releases for different distributions.
