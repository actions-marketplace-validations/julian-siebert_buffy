FROM debian:trixie-slim
ENV DEBIAN_FRONTEND=noninteractive

# base tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    git \
    gnupg \
    pkg-config \
    libssl-dev \
    build-essential \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# protoc
ARG PROTOC_VERSION=29.3
RUN curl -fsSL -o /tmp/protoc.zip \
      "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-x86_64.zip" \
    && mkdir -p /opt/protoc \
    && (cd /opt/protoc && unzip -q /tmp/protoc.zip) \
    && ln -s /opt/protoc/bin/protoc /usr/local/bin/protoc \
    && rm /tmp/protoc.zip

# go + protoc plugins for go
ARG GO_VERSION=1.23.4
RUN curl -fsSL "https://go.dev/dl/go${GO_VERSION}.linux-amd64.tar.gz" \
    | tar -C /usr/local -xzf - \
 && ln -s /usr/local/go/bin/go /usr/local/bin/go \
 && ln -s /usr/local/go/bin/gofmt /usr/local/bin/gofmt
ENV GOPATH=/go
ENV PATH=$PATH:/go/bin
RUN go install google.golang.org/protobuf/cmd/protoc-gen-go@latest \
 && go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

# rust + cargo plugins
ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile minimal \
 && cargo install \
      protoc-gen-prost \
      protoc-gen-prost-crate \
      protoc-gen-tonic

# maven + gpg
RUN apt-get update && apt-get install -y --no-install-recommends \
    maven \
    openjdk-25-jdk-headless \
    && rm -rf /var/lib/apt/lists/*

RUN echo "export JAVA_HOME=\$(readlink -f /usr/bin/java | sed 's|/bin/java||')" >> /etc/profile.d/java.sh
ENV JAVA_HOME=/usr/lib/jvm/java-25-openjdk-amd64


# node + npm + ts-proto + typescript
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
 && apt-get install -y --no-install-recommends nodejs \
 && rm -rf /var/lib/apt/lists/* \
 && npm install -g ts-proto typescript

# python + grpcio-tools + build + twine
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 python3-pip python3-venv \
    && rm -rf /var/lib/apt/lists/* \
 && pip3 install --no-cache-dir --break-system-packages \
      grpcio-tools build twine

ARG BUFFY_VERSION
ARG TARGETARCH

RUN arch="${TARGETARCH}" \
  && if [ -z "$arch" ]; then \
       case "$(uname -m)" in \
         x86_64)  arch=amd64 ;; \
         aarch64) arch=arm64 ;; \
         *) echo "unsupported uname -m: $(uname -m)" && exit 1 ;; \
       esac; \
     fi \
  && case "$arch" in \
       amd64) target=x86_64-unknown-linux-gnu ;; \
       arm64) target=aarch64-unknown-linux-gnu ;; \
       *) echo "unsupported arch: $arch" && exit 1 ;; \
     esac \
  && curl -fsSL "https://pkgs.julian-siebert.de/buffy/${BUFFY_VERSION}/${target}/buffy" \
     -o /usr/local/bin/buffy \
  && chmod +x /usr/local/bin/buffy


WORKDIR /work
ENTRYPOINT ["/usr/local/bin/buffy"]
