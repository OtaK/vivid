#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_language(0x0000).set_icon("assets/vivid.ico");

    res.compile().unwrap();
}

#[cfg(not(windows))]
fn main() {
    panic!("This product cannot be built on other platforms than Windows!")
}
