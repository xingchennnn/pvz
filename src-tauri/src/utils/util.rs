/** &str 转 *const u16 */
pub fn str_to_pcwstr(s: &str) -> Vec<u16> {
    let result = s
        .to_string()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    result
}