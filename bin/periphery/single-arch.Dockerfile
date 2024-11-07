## Assumes the latest binaries for the required arch are already built (by binaries.Dockerfile).
## Sets up the necessary runtime container dependencies for Komodo Periphery.

ARG REGISTRY_AND_NAMESPACE=ghcr.io/mbecker20
ARG IMAGE_TAG=latest
ARG BINARIES=${REGISTRY_AND_NAMESPACE}/binaries:${IMAGE_TAG}

# This is required to work with COPY --from
FROM ${BINARIES} AS binaries

FROM debian:bullseye-slim

COPY ./bin/periphery/debian-deps.sh .
RUN sh ./debian-deps.sh && rm ./debian-deps.sh

WORKDIR /app
COPY --from=binaries /periphery /usr/local/bin/periphery

EXPOSE 8120

LABEL org.opencontainers.image.source=https://github.com/mbecker20/komodo
LABEL org.opencontainers.image.description="Komodo Periphery"
LABEL org.opencontainers.image.licenses=GPL-3.0

CMD [ "periphery" ]