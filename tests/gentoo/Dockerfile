# name the portage image
FROM gentoo/portage:latest as portage

# based on stage3 image
FROM gentoo/stage3:latest as base

# copy the entire portage volume in
COPY --from=portage /var/db/repos/gentoo /var/db/repos/gentoo

# install rust
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

COPY tests/gentoo/entrypoint.sh ./

RUN chmod +x entrypoint.sh

CMD ["./entrypoint.sh"]