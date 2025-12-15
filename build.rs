#[cfg(windows)]
extern crate winres;

fn main() {
    #[cfg(windows)]
    winres::WindowsResource::new()
        .set_icon("meta/io.github.pieterdd.RcloneShuttle.ico")
        .compile()
        .unwrap();

    relm4_icons_build::bundle_icons(
        "icon_names.rs",
        None,
        None,
        None::<&str>,
        [
            "left-large",
            "right-large",
            "up-large",
            "arrow-circular-top-right",
            "warning-outline",
            "plus",
            "folder-filled",
            "paper-filled",
            "check-round-outline",
            "error-outline",
            "padlock2",
            "menu",
            "info-outline",
            "minus-circle-filled",
            "brush",
        ],
    );
}
