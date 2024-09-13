FROM ubuntu:latest
RUN apt-get update && apt-get install -y git libgtk-4-dev build-essential libglib2.0-dev libadwaita-1-dev cargo
WORKDIR /build
COPY . /build
RUN cargo build