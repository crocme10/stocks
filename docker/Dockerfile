ARG RUST_VERSION
ARG DEBIAN_VERSION

FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} as builder

WORKDIR /home

ARG DEBIAN_VERSION

RUN USER=root cargo new stocks

WORKDIR /home/stocks

COPY Cargo.toml Cargo.lock ./

RUN echo "// dummy file" > src/lib.rs
RUN mkdir tests
RUN echo "// dummy file" > tests/currency.rs

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src
COPY ./docker ./docker
COPY ./config ./config

RUN rm ./target/release/deps/stocks*
RUN rm ./target/release/deps/libstocks*

RUN cargo build --release

ARG DEBIAN_VERSION

FROM debian:${DEBIAN_VERSION}-slim

ARG DEBIAN_VERSION

RUN if [ "${DEBIAN_VERSION}" = "buster" ]; then \
  apt-get update \
    && apt-get install -y netcat libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*; \
elif [ "${DEBIAN_VERSION}" = "stretch" ]; then \
  apt-get update \
    && apt-get install -y netcat libssl1.0-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*; \
else \
  echo "Unsupported debian version '$DEBIAN_VERSION'"; \
fi

ENV TZ=Etc/UTC \
    RUN_MODE=development \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p /opt/stocks \
    && mkdir -p /etc/opt/stocks

COPY --from=builder /home/stocks/target/release/service /opt/stocks/service
COPY --from=builder /home/stocks/config /etc/opt/stocks
COPY --from=builder /home/stocks/docker/entrypoint.sh /opt/stocks/entrypoint.sh

RUN chown -R $APP_USER:$APP_USER /opt/stocks
RUN mkdir /var/log/stocks
RUN chown -R $APP_USER:$APP_USER /var/log/stocks

USER $APP_USER
WORKDIR /opt/stocks

CMD ["./entrypoint.sh"]
