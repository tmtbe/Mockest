FROM --platform=$TARGETPLATFORM envoyproxy/envoy:v1.21.2 as builder-envoy
FROM --platform=$TARGETPLATFORM rust:1.61.0 as builder-rust
WORKDIR /app
COPY ./intercept .
RUN rustup target add wasm32-unknown-unknown
RUN cargo build --target wasm32-unknown-unknown --release

FROM --platform=$TARGETPLATFORM golang:1.17-alpine as builder-golang
WORKDIR /app
COPY ./cmd .
RUN go mod tidy \
    && CGO_ENABLED=0 GOOS=linux go build -o proxy

FROM --platform=$TARGETPLATFORM ubuntu:jammy
RUN apt-get update && \
  apt-get install --no-install-recommends -y \
  ca-certificates \
  curl \
  iptables \
  iproute2 \
  iputils-ping \
  net-tools \
  lsof \
  sudo \
  && update-ca-certificates \
  && apt-get upgrade -y \
  && apt-get clean \
  && rm -rf  /var/log/*log /var/lib/apt/lists/* /var/log/apt/* /var/lib/dpkg/*-old /var/cache/debconf/*-old \
  && update-alternatives --set iptables /usr/sbin/iptables-legacy \
  && update-alternatives --set ip6tables /usr/sbin/ip6tables-legacy
RUN useradd -m --uid 1987 sidecar && \
   echo "sidecar ALL=NOPASSWD: ALL" >> /etc/sudoers

COPY --from=builder-rust app/target/wasm32-unknown-unknown/release/intercept.wasm /home
COPY --from=builder-golang app/proxy /home
COPY --from=builder-envoy  usr/local/bin/envoy /home
COPY resource /home
USER sidecar
WORKDIR /home
ENTRYPOINT ["./proxy"]
