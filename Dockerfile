# Builder stage
FROM debian:bookworm-slim AS builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    git \
    make

# Install Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . .

RUN cargo build --release
RUN mkdir -p /root/.local/bin
RUN make install

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    fzf \
    bat \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /root/.local/bin/snix /usr/local/bin/snix

ENV PATH="/usr/local/bin:${PATH}"
ENV TERM=xterm-256color

CMD ["snix"]