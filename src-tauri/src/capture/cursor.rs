/// Cursor capture and rendering utilities.
/// Windows Graphics Capture API can capture the cursor natively as part of the frame.
/// We enable this by default and do NOT draw an additional software cursor.
/// This module handles cursor state tracking for informational purposes only.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorState {
    pub x: i32,
    pub y: i32,
    pub visible: bool,
    pub cursor_type: String,
}

/// Get current cursor state for diagnostics/metadata.
/// NOTE: We do NOT use this to render the cursor into frames;
/// that is handled by the WGC capture API's built-in cursor rendering.
pub fn get_cursor_state() -> CursorState {
    #[cfg(windows)]
    {
        get_cursor_state_windows()
    }
    #[cfg(not(windows))]
    {
        CursorState { x: 0, y: 0, visible: true, cursor_type: "arrow".into() }
    }
}

#[cfg(windows)]
fn get_cursor_state_windows() -> CursorState {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetCursorInfo, CURSORINFO, CURSOR_SHOWING,
    };
    use windows::Win32::Foundation::POINT;

    let mut info = CURSORINFO {
        cbSize: std::mem::size_of::<CURSORINFO>() as u32,
        flags: Default::default(),
        hCursor: Default::default(),
        ptScreenPos: POINT { x: 0, y: 0 },
    };

    let visible = unsafe {
        GetCursorInfo(&mut info).is_ok() && (info.flags.0 & CURSOR_SHOWING.0) != 0
    };

    CursorState {
        x: info.ptScreenPos.x,
        y: info.ptScreenPos.y,
        visible,
        cursor_type: "arrow".into(), // Would need LoadCursor/GetIconInfo for exact type
    }
}
