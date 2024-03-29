# base image
FROM ubuntu:jammy

ARG TARGETARCH
ARG VERSION="0.8.0" x-release-please-version

# https://github.com/opencontainers/image-spec/blob/main/annotations.md#pre-defined-annotation-keys
LABEL org.opencontainers.image.base.name="ubuntu:jammy"
LABEL org.opencontainers.image.url="https://github.com/fluencelabs/capacity-commitment-prover"
LABEL org.opencontainers.image.vendor="fluencelabs"
LABEL org.opencontainers.image.authors="fluencelabs"
LABEL org.opencontainers.image.title="ccp"
LABEL org.opencontainers.image.description="ccp runs on the capacity provider to prove that capacity is allocated"
LABEL org.opencontainers.image.version=${VERSION}

# install necessary packages
RUN \
  apt-get update && \
  apt-get install -y --no-install-recommends \
    libhwloc-dev \
    gosu \
    ca-certificates \
  	jq \
  	curl && \
  apt-get clean && \
  rm -rf \
  	/tmp/* \
  	/var/lib/apt/lists/* \
  	/var/tmp/*

# copy binary
COPY ./binaries/ccp-${TARGETARCH}/ccp /usr/bin/ccp
RUN chmod +x /usr/bin/ccp
# copy default ccp config
COPY Config.default.toml /fluence/Config.toml
# copy entrypoint script
COPY entrypoint.sh /entrypoint.sh

VOLUME /fluence/data

ENTRYPOINT ["/entrypoint.sh"]
