<img src="meta/logo-with-text.png" alt="Rclone Shuttle logo" /><br />

GTK4 frontend for Rclone to upload files to [any supported](https://rclone.org/overview/) cloud storage provider or storage protocol.

<a href="https://flathub.org/apps/io.github.pieterdd.RcloneShuttle"><img width="170" alt="Download on Flathub" src="https://flathub.org/api/badge?locale=en"/></a>

Rclone Shuttle can:

- Upload files via drag and drop
- Rename, move, copy and delete files/folders
- Open remote files locally via double click
- Permanently save a local copy of specific files/folders
- Handle encrypted Rclone configuration files

![Screenshot](meta/screenshots/browser.png)

Some of Rclone's supported cloud storage providers and protocols:

- Amazon S3 and API-compatible derivatives
- Dropbox
- FTP
- Google Cloud Storage
- Google Drive
- Google Photos
- OneDrive
- Proton Drive
- SFTP
- SMB (Samba)
- WebDAV (works with Nextcloud)

## Install

### Linux
[Flathub](https://flathub.org/apps/io.github.pieterdd.RcloneShuttle) is our official distribution mechanism for Linux. We have no plans to offer PPA/COPR repos or other distribution-specific update channels. Your distro might still have an [unofficial package](https://repology.org/project/rclone-shuttle/versions).

### Building from source
Since Rclone Shuttle is written in Rust, you can generate a release build for your architecture and OS by checking out the repo and running `cargo build --release`. You will need [GTK's development kit](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation.html) and Rclone v1.66 or up.
