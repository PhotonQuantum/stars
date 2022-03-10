# Based on tumbleweed image
FROM opensuse/tumbleweed:latest AS base

# Install the required packages
RUN zypper -n in -t pattern devel_basis && \
    zypper -n in curl openssl-devel

# Install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:$PATH"

# install cargo-chef
RUN cargo install cargo-chef --locked

WORKDIR /app

FROM base AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json

FROM base AS runner

COPY --from=planner /app/recipe.json ./

RUN cargo chef cook --recipe-path recipe.json

COPY . .

COPY tests/opensuse/entrypoint.sh ./

RUN chmod +x entrypoint.sh

CMD ["./entrypoint.sh"]