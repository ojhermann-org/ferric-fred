{
  description = "ferric-fred — a strongly-typed Rust client for FRED, plus a CLI and MCP server";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Recent stable toolchain (ADR-0007: track stable, no pinned MSRV yet).
        # Components cover editor + lint/format tooling out of the box.
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
            "rustfmt"
          ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          # No OpenSSL / pkg-config here on purpose: ADR-0003 chose rustls-tls,
          # so the HTTP stack has no system TLS dependency.
          packages = [
            rustToolchain
            pkgs.cargo-nextest
            pkgs.cargo-deny
            pkgs.bacon # background `cargo check`/clippy/test runner
            pkgs.infisical # CLI: inject secret values (e.g. FRED_API_KEY) via direnv
            pkgs.gitleaks # secret scanner for the pre-commit guard (.githooks/pre-commit)
          ];

          env.RUST_BACKTRACE = "1";

          shellHook = ''
            echo "ferric-fred dev shell — $(rustc --version)"
          '';
        };

        # `nix fmt` formats the Nix files in this repo.
        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
