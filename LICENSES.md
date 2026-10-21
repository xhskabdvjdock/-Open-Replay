# Open Replay - Licenses & Dependencies

## Application License

Open Replay is proprietary software. All rights reserved.

## Third-Party Dependencies

### Rust Backend

| Dependency | Version | License | Purpose |
|---|---|---|---|
| tauri | 2.x | MIT/Apache-2.0 | Application framework |
| serde | 1.x | MIT/Apache-2.0 | Serialization |
| serde_json | 1.x | MIT/Apache-2.0 | JSON serialization |
| tokio | 1.x | MIT | Async runtime |
| windows | 0.61 | MIT/Apache-2.0 | Windows API bindings |
| windows-core | 0.61 | MIT/Apache-2.0 | Windows core types |
| thiserror | 2.x | MIT/Apache-2.0 | Error derive macros |
| anyhow | 1.x | MIT/Apache-2.0 | Error handling |
| tracing | 0.1 | MIT | Structured logging |
| tracing-subscriber | 0.3 | MIT | Log formatting |
| tracing-appender | 0.2 | MIT | Log file output |
| rusqlite | 0.34 | MIT | SQLite database |
| chrono | 0.4 | MIT/Apache-2.0 | Date/time handling |
| uuid | 1.x | MIT/Apache-2.0 | Unique identifiers |
| dirs | 6.x | MIT/Apache-2.0 | Platform directories |
| image | 0.25 | MIT/Apache-2.0 | Image processing |
| parking_lot | 0.12 | MIT/Apache-2.0 | Synchronization primitives |
| crossbeam | 0.8 | MIT/Apache-2.0 | Concurrent utilities |
| bytemuck | 1.x | MIT/Apache-2.0/Zlib | Byte casting |
| bytes | 1.x | MIT | Byte buffers |
| toml | 0.8 | MIT/Apache-2.0 | TOML config parsing |

### Frontend

| Dependency | Version | License | Purpose |
|---|---|---|---|
| react | 19.x | MIT | UI framework |
| react-dom | 19.x | MIT | React DOM rendering |
| react-router-dom | 7.x | MIT | Client-side routing |
| @tauri-apps/api | 2.x | MIT/Apache-2.0 | Tauri API bindings |
| lucide-react | latest | ISC | Icon library |
| i18next | latest | MIT | Internationalization |
| react-i18next | latest | MIT | React i18n integration |
| typescript | 6.x | Apache-2.0 | Type checking |
| vite | 8.x | MIT | Build tool |

### System APIs (Windows)

| API | Purpose | License |
|---|---|---|
| Windows Graphics Capture | Screen capture | Part of Windows OS |
| WASAPI | Audio capture | Part of Windows OS |
| Media Foundation | Video encoding/decoding | Part of Windows OS |
| Direct3D 11 | GPU acceleration | Part of Windows OS |
| DXGI | Display management | Part of Windows OS |

### Fonts

| Font | License | Notes |
|---|---|---|
| Thmanyah | Check font license | Arabic UI font - must be obtained with proper license |
| Segoe UI | Microsoft EULA | Fallback font (system-installed) |

## Notes

- This application does not bundle FFmpeg or any GPL-licensed software.
- All video encoding uses Windows built-in Media Foundation APIs.
- The `rusqlite` crate bundles SQLite (public domain).
- No external services or APIs are used at runtime.
