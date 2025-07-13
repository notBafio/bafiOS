FROM rust

COPY . /root/bafiOS

RUN apt-get -y update
RUN apt-get -y install mtools dosfstools
RUN rustup component add rust-src --toolchain nightly-2025-01-01-x86_64-unknown-linux-gnu

RUN apt-get -y install qemu-system-x86_64

WORKDIR /root/bafiOS
CMD ["make", "all"]