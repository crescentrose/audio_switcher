//! Various helper methods pertaining to Bluetooth management.

/// Helper method to convert the helpfully null-padded UTF-16 string given to us by the Windows API into a proper Rust [String].
///
/// # Arguments
///
/// * `slice` - A slice of 16-bit integers representing a (hopefully) valid, null-padded UTF-16 string.
///
/// # Example
///
/// ```rust
///     use audio_switcher::bluetooth::util::u16_slice_to_string;
///
///     let hello: [u16; 8] = [0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x0, 0x0, 0x0];
///     assert_eq!(u16_slice_to_string(hello.as_slice()), String::from("hello"));
/// ```
pub fn u16_slice_to_string(slice: &[u16]) -> String {
    String::from_utf16_lossy(slice)
        .trim_matches(char::from(0))
        .to_string()
}
