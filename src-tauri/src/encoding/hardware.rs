/// Hardware encoder stubs - actual implementation via Media Foundation MFT.
/// Hardware encoding happens inside the Windows encoder MFT, which is
/// selected automatically when quality/performance requirements are met.
/// The muxer uses Media Foundation Sink Writer which handles encoder selection.

pub struct HardwareEncoder;

impl HardwareEncoder {
    pub fn is_available() -> bool {
        #[cfg(windows)]
        {
            // Check if any hardware MFT encoder is available
            super::encoder::enumerate_encoders()
                .iter()
                .any(|e| e.encoder_type == "hardware" && e.is_available)
        }
        #[cfg(not(windows))]
        false
    }
}
