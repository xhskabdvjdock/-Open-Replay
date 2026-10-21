/// Software H.264 encoder via Windows Media Foundation.
/// Used as fallback when hardware encoding is unavailable.

pub struct SoftwareEncoder;

impl SoftwareEncoder {
    /// Software H.264 encoding is always available on Windows 10+
    pub fn is_available() -> bool {
        cfg!(windows)
    }
}
