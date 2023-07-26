FROM ubuntu:23.04

ENV LANG=C.UTF-8

# Support various rvm, nvm etc stuff which requires executing profile scripts (-l)
SHELL ["/bin/bash", "-lc"]
CMD ["/bin/bash", "-l"]

# Set debconf to run non-interactively
RUN echo 'debconf debconf/frontend select Noninteractive' | debconf-set-selections

RUN apt-get update && apt-get install -y apt-utils apt-transport-https software-properties-common

# Newest git
RUN apt-add-repository ppa:git-core/ppa -y && apt-get update


RUN set -ex -o pipefail && apt-get install -y \
    # Useful utilities \
    curl unzip wget socat man-db rsync moreutils vim lsof libssl-dev \
    # C/C++ \
    build-essential cmake g++ m4 \
    # R \
    r-base r-base-dev \
    # JVM \
    openjdk-8-jre-headless openjdk-11-jdk-headless openjdk-17-jdk-headless maven ant clojure scala \
    # Python 3 \
    python3-matplotlib python3-numpy python3-pip python3-scipy python3-pandas python3-dev pipenv \
    # Python 2 \
    python2-dev python-pip-whl \
    && \
    # Check Python \
    python3 --version && python2 --version && pip3 --version && \
    # Rust \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

## Nodejs, npm, yarn
RUN set -ex -o pipefail &&  \
    curl -fsSL https://deb.nodesource.com/setup_16.x | bash - && \
    curl -fsSL https://dl.yarnpkg.com/debian/pubkey.gpg | gpg --dearmor | tee /usr/share/keyrings/yarnkey.gpg >/dev/null && \
    echo "deb [signed-by=/usr/share/keyrings/yarnkey.gpg] https://dl.yarnpkg.com/debian stable main" | tee /etc/apt/sources.list.d/yarn.list && \
    apt-get update && apt-get install -y nodejs yarn

## Rust tools
RUN cargo install cargo-watch && \
    cargo install wasm-pack --features curl/static-curl --version="=0.10.2"
