#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    winres::WindowsResource::new()
        .set_icon("meta/io.github.pieterdd.RcloneShuttle.ico")
        .compile()
        .unwrap();
}

#[cfg(not(windows))]
fn main() {}
