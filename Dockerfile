FROM archlinux:latest
RUN pacman -Syu --noconfirm gtk4 gcc glib2-devel libadwaita rust pkgconf
WORKDIR /build
COPY . /build
RUN cargo build
