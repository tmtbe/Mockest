FROM ubuntu:jammy
RUN apt-get update && \
  apt-get install --no-install-recommends -y \
  ca-certificates \
  curl \
  iptables \
  iproute2 \
  iputils-ping \
  knot-dnsutils \
  netcat \
  tcpdump \
  conntrack \
  bsdmainutils \
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
COPY ./envoy /home/sidecar
COPY ./run.sh /home/sidecar
COPY ./envoy.yaml /home/sidecar
USER sidecar
WORKDIR /home/sidecar
CMD sh ./run.sh
