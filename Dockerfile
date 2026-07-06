# Container image for the ferric-fred MCP server (`fred-mcp`).
#
# Multi-stage: compile the release binary with the Rust toolchain, then copy
# just that binary into a slim runtime. The server speaks MCP over stdio, so run
# it with stdin/stdout attached:  docker run -i -e FRED_API_KEY=... <image>
#
# FRED_API_KEY must be set for the server to *start*. The MCP handshake
# (initialize / tools/list) never calls FRED, so the placeholder below is enough
# to boot and introspect (this is what registry/index checks exercise). Override
# it with a real key (free at https://fredaccount.stlouisfed.org/apikeys) to
# actually query FRED.

FROM rust:1-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release -p ferric-fred-mcp

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/fred-mcp /usr/local/bin/fred-mcp
ENV FRED_API_KEY=replace-with-your-fred-api-key
ENTRYPOINT ["fred-mcp"]
