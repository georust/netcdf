# Mirroring travis build (xenial 16.04) for now...
FROM ubuntu:16.04

RUN apt-get -y update && apt-get -y install \
    libhdf5-serial-dev \
    netcdf-bin \
    libnetcdf-dev \
    curl

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y

ENV PATH=/root/.cargo/bin:$PATH

ADD . /code

WORKDIR /code

CMD cargo build --verbose && cargo test -j1 --verbose
