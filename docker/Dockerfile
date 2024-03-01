# base image
FROM ubuntu:jammy

ARG TARGETARCH

# https://github.com/opencontainers/image-spec/blob/main/annotations.md#pre-defined-annotation-keys
LABEL org.opencontainers.image.base.name="ubuntu:jammy"
LABEL org.opencontainers.image.url="https://github.com/fluencelabs/capacity-commitment-prover"
LABEL org.opencontainers.image.vendor="fluencelabs"
LABEL org.opencontainers.image.authors="fluencelabs"
LABEL org.opencontainers.image.title="ccp"
LABEL org.opencontainers.image.description="ccp runs on the capacity provider to prove that capacity is allocated."

# install necessary packages
RUN \
  apt-get update && \
  apt-get install -y --no-install-recommends \
    gosu \
    ca-certificates \
  	jq \
  	curl \
  apt-get clean && \
  rm -rf \
  	/tmp/* \
  	/var/lib/apt/lists/* \
  	/var/tmp/*

# copy binary
COPY ./binaries/ccp-${TARGETARCH}/nox /usr/bin/ccp
RUN chmod +x /usr/bin/ccp
# copy entrypoint script
COPY entrypoint.sh /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
