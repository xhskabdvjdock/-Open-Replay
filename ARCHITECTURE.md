# Open Replay - Architecture Document

## Overview

Open Replay is a desktop application for Windows that continuously captures the screen in a ring buffer and allows the user to save the last N seconds as a video clip with a single hotkey press.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend (React)                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────────┐│
│  │ HomePage  │ │ Library  │ │  Editor  │ │      Settings       ││
│  │  Status   │ │  Clips   │ │ Player   │ │  Replay/Capture/    ││
│  │  Buffer   │ │  Search  │ │ Timeline │ │  Audio/Video/...    ││
│  │  Quick    │ │  Grid/   │ │ Trim     │ │                     ││
│  │  Settings │ │  List    │ │ Export   │ │                     ││
│  └──────────┘ └──────────┘ └──────────┘ └──────────────────────┘│
│                          Tauri IPC                               │
└──────────────────────────┬──────────────────────────────────────┘
                           │ invoke() / events
┌──────────────────────────┴──────────────────────────────────────┐
│                      Backend (Rust/Tauri)                        │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                     Command Layer                           ││
│  │  Tauri commands that bridge frontend requests to engines    ││
│  └────────┬────────────────┬──────────────┬───────────────────┘│
│           │                │              │                     │
│  ┌────────▼────────┐ ┌────▼─────┐ ┌─────▼──────┐             │
│  │  Replay Engine  │ │ Library  │ │  Export    │              │
│  │                 │ │ Manager  │ │  Engine    │              │
│  │  ┌───────────┐  │ │          │ │            │              │
│  │  │  Screen   │  │ │ SQLite   │ │ Trimming   │              │
│  │  │  Capture  │  │ │ DB       │ │ Re-encode  │              │
│  │  │  (WGC)    │  │ │          │ │ MP4 Mux    │              │
│  │  └─────┬─────┘  │ └──────────┘ └────────────┘              │
│  │        │        │                                           │
│  │  ┌─────▼─────┐  │  ┌───────────┐  ┌────────────┐          │
│  │  │  Encoder  │  │  │ Settings  │  │ Shortcuts  │           │
│  │  │  (MFT)    │  │  │  (TOML)   │  │ (Global)   │           │
│  │  │  HW / SW  │  │  │           │  │            │           │
│  │  └─────┬─────┘  │  └───────────┘  └────────────┘          │
│  │        │        │                                           │
│  │  ┌─────▼─────┐  │  ┌───────────────────────────┐          │
│  │  │  Replay   │  │  │     Audio Engine          │           │
│  │  │  Buffer   │  │  │  ┌─────────┐ ┌─────────┐ │           │
│  │  │ (Ring)    │  │  │  │Loopback │ │   Mic   │ │           │
│  │  │           │  │  │  │(WASAPI) │ │(WASAPI) │ │           │
│  │  └───────────┘  │  │  └─────────┘ └─────────┘ │           │
│  └─────────────────┘  └───────────────────────────┘           │
│                                                                │
│  ┌────────────────────────────────────────────────────────────┐│
│  │                    System Layer                            ││
│  │  D3D11 Device │ COM Init │ Tray │ Logging │ Diagnostics   ││
│  └────────────────────────────────────────────────────────────┘│
└────────────────────────────────────────────────────────────────┘
```

## Core Design Decisions

### 1. Capture Pipeline: Windows Graphics Capture API

**Chosen:** Windows Graphics Capture (WGC) API
**Reason:** Native Windows API available since Windows 10 1903. Captures at GPU level, includes cursor natively, handles DPI scaling, supports both monitor and window capture. Lower overhead than DXGI Desktop Duplication for most use cases.

**Alternative considered:** DXGI Desktop Duplication
- Better for exclusive fullscreen games
- More complex cursor handling
- Available on older Windows versions
- Kept as a potential fallback but not implemented in v1

### 2. Encoding: Media Foundation Transform (MFT)

**Chosen:** Media Foundation for both encoding and muxing
**Reason:** 
- Built into Windows, no external dependencies
- Supports hardware encoding (NVENC, AMF, QSV) through MFT
- Can accept D3D11 textures directly (zero-copy on GPU)
- Produces standard MP4/H.264/AAC output
- No licensing concerns (unlike FFmpeg/x264)

**Fallback:** Software H.264 encoder via Media Foundation (always available on Windows 10+)

### 3. Replay Buffer: In-Memory Encoded Packet Ring Buffer

**Chosen:** Option 1 - Ring buffer of encoded packets
**Reason:**
- Lower memory usage than raw frames (H.264 compression ~100:1)
- Faster save (just mux existing packets, no re-encoding needed)
- Consistent memory usage regardless of save frequency
- Keyframe-aligned extraction ensures clean clip boundaries

**Implementation:**
- VecDeque of `EncodedPacket` structs
- Each packet has: data, timestamp, is_keyframe flag, stream type
- Eviction by timestamp (remove packets older than buffer duration)
- Save: find nearest keyframe to start, extract packets through end, mux into MP4
- Thread-safe with parking_lot::Mutex
- Separate save path doesn't block new packet insertion

### 4. Audio: WASAPI

**Chosen:** WASAPI for both loopback (system audio) and input (microphone)
**Reason:** Native Windows audio API, low latency, supports all audio devices, allows device selection.

**Mixing:** When both sources are enabled, mix into a single AAC stream. Optional separate tracks as advanced feature.

### 5. Frontend: React + TypeScript in Tauri 2

Standard Tauri 2 architecture. Frontend handles UI only - no capture, encoding, or heavy processing in the webview.

## Data Flow

### Capture Flow
```
Monitor → WGC Frame Pool → D3D11 Texture → MFT Encoder → H.264 Packets → Ring Buffer
                                                                              ↓
Audio Device → WASAPI → PCM Samples → AAC Encoder → AAC Packets → Ring Buffer
```

### Save Flow
```
User presses hotkey → Ring Buffer extract (keyframe-aligned) → MFT Sink Writer → MP4 File
                                                                                    ↓
                                                                              Library DB update
                                                                              Thumbnail generation
                                                                              Frontend notification
```

### Export Flow
```
MP4 Source → MF Source Reader → Decode → Trim → Re-encode (optional) → MF Sink Writer → Output MP4
```

## Threading Model

| Thread | Purpose | Priority |
|--------|---------|----------|
| Main (Tauri) | UI, IPC, app lifecycle | Normal |
| Capture Thread | WGC frame acquisition | Above Normal |
| Encoder Thread | H.264 encoding | Above Normal |
| Audio Loopback Thread | System audio capture | Above Normal |
| Audio Mic Thread | Microphone capture | Above Normal |
| Save Thread | Mux packets to MP4 | Normal |
| Export Thread | Re-encode and mux | Below Normal |

## File Storage

```
%APPDATA%/com.openreplay.app/
├── settings.toml          # User settings
├── library.db             # SQLite clip database
├── logs/                  # Application logs
│   └── open-replay.log
├── clips/                 # Saved replay clips
│   ├── clip_xxxx.mp4
│   └── ...
├── thumbnails/            # Clip thumbnails
│   ├── clip_xxxx.jpg
│   └── ...
└── temp/                  # Temporary files (cleaned on startup)
```

## Error Recovery

1. **Encoder failure:** Log error, attempt restart with software encoder, notify user
2. **Device lost:** Pause capture, attempt reinitialization, resume or notify
3. **Disk full:** Stop buffer writes, notify user, preserve existing clips
4. **Crash recovery:** On startup, clean temp files, verify library integrity
5. **Save failure:** Preserve buffer data, show error with details, allow retry

## Security Model

- All data stays local
- No network access required
- No telemetry
- Minimal OS permissions (screen capture, audio, file system)
- Tauri capability-based security
- No code execution from user data
