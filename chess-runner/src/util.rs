pub fn add_exe_on_windows<S: AsRef<str>>(s: S) -> String {
    if cfg!(windows) {
        format!("{}.exe", s.as_ref().replace("/", "\\"))
    } else {
        s.as_ref().to_string()
    }
}
