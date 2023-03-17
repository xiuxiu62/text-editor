#[inline]
pub const fn newline() -> &'static str {
    if cfg!(windows) {
        return "\r\n";
    }

    "\n"
}
