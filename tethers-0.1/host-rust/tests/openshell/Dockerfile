FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install --no-install-recommends --yes curl iproute2 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 1000 sandbox \
    && useradd --uid 1000 --gid 1000 --create-home --shell /bin/sh sandbox \
    && mkdir -p /sandbox/outbox/approved /sandbox/outbox/forbidden \
    && chown -R sandbox:sandbox /sandbox

USER sandbox

CMD ["/usr/bin/tail", "-f", "/dev/null"]
