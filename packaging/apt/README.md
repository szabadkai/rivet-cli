# Apt Packaging Plan for Rivet CLI

This document outlines how to distribute Rivet via apt for Debian/Ubuntu users, plus any code and CI changes needed.

## Goals
- Provide `apt install rivet` (or `rivet-cli`) on Debian/Ubuntu.
- Offer signed packages for `amd64` and `arm64` with a simple, stable repo URL.
- Keep maintenance low by integrating into the existing release flow.

## Distribution Options

- Cloud-hosted repo (recommended for simplicity)
  - Cloudsmith or Packagecloud: managed apt repos, GPG signing, per‑distro channels.
  - Pros: fastest setup, solid tooling, good docs. Cons: paid for higher usage.

- Self-hosted on GitHub Pages + aptly/reprepro
  - Use GitHub Actions to build `.deb`, generate repo metadata via `aptly`, publish to `gh-pages`.
  - Pros: low cost, full control. Cons: more moving parts (key mgmt, repo structure).

We can support either; both share the same `.deb` build step.

## Build .deb with cargo-deb

Use `cargo-deb` to produce Debian packages directly from Cargo metadata.

- Install locally: `cargo install cargo-deb`
- Build: `cargo deb --target x86_64-unknown-linux-gnu` (and `aarch64-unknown-linux-gnu`)

Add packaging metadata to `Cargo.toml`:

```toml
[package.metadata.deb]
# If the Debian package should be named rivet-cli to avoid collisions
name = "rivet-cli"              # or "rivet" if available in Debian namespace
maintainer = "<Your Name> <you@example.com>"
license-file = ["LICENSE"]
section = "utils"
priority = "optional"
# Minimal runtime deps (Rust static binaries typically have none)
depends = []
description = "API testing that lives in git"
# Install paths and extra assets (see Completions/Man Page sections)
assets = [
  ["target/release/rivet", "/usr/bin/rivet", "755"],
  # Bash/Zsh/Fish completions (optional but recommended)
  ["packaging/assets/completions/rivet.bash", "/usr/share/bash-completion/completions/rivet", "644"],
  ["packaging/assets/completions/_rivet", "/usr/share/zsh/vendor-completions/_rivet", "644"],
  ["packaging/assets/completions/rivet.fish", "/usr/share/fish/vendor_completions.d/rivet.fish", "644"],
  # Man page (optional but recommended)
  ["packaging/assets/man/rivet.1", "/usr/share/man/man1/rivet.1", "644"],
]
# If the binary is named "rivet" but package is "rivet-cli"
conflicts = ["rivet"]
replaces  = ["rivet"]
```

Notes:
- `cargo-deb` reads the top-level crate metadata; keep `description`, `license`, `repository` up to date.
- If you stick with package name `rivet`, drop `conflicts/replaces`.

## Code Changes (Recommended)

Not strictly required for packaging, but improves Linux UX and packaging quality.

- Shell completions: add `clap_complete` and generate Bash/Zsh/Fish files.
  - Option A: subcommand (`rivet completions <shell>`) that writes to stdout.
  - Option B: generate at release time and include in the `.deb` via `assets`.
- Man page: add `clap_mangen` to generate a `rivet.1` man page from clap definitions.
- Version/ident: already provided by clap’s `--version`. Ensure it prints SemVer only.
- Filesystem paths: current defaults to `~/.rivet/` are fine; Debian policy does not require system configs.

Example code additions (high level):
- Dependency additions in `Cargo.toml`:
  - `clap_complete = "*"`
  - `clap_mangen = "*"`
- In `main.rs` add hidden subcommands:
  - `rivet completions <bash|zsh|fish>`
  - `rivet man` (outputs `rivet.1`)
- For packaging, generate these during CI and place under `packaging/assets/`.

## CI Changes (GitHub Actions)

Add a workflow that:

1) Builds binaries and `.deb` packages for Linux targets.
- Matrix: `ubuntu-22.04`, arch `amd64` and `arm64`.
- Install Rust toolchains and `cargo-deb`.
- Build release binaries and run `cargo deb` per arch.

2) Signs and publishes.
- If using Cloudsmith/Packagecloud: use their action to push `.deb` files and upload GPG public key.
- If self-hosting: use `aptly` to create repo, sign with GPG (`gpg --batch --pinentry-mode loopback`), publish to `gh-pages`.

Example workflow snippet (outline):

```yaml
name: release-linux-deb
on:
  push:
    tags: [ 'v*' ]
jobs:
  build:
    runs-on: ubuntu-22.04
    strategy:
      matrix:
        target: [ x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu ]
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}
          profile: minimal
      - name: Install cargo-deb
        run: cargo install cargo-deb
      - name: Build release
        run: cargo build --release --target ${{ matrix.target }}
      - name: Build .deb
        run: cargo deb --target ${{ matrix.target }} --no-build
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: deb-${{ matrix.target }}
          path: target/${{ matrix.target }}/debian/*.deb
```

Publishing (choose one):
- Cloudsmith: `cloudsmith push deb your-org/your-repo/any-distro/any-version *.deb`
- Packagecloud: `package_cloud push user/repo/ubuntu/jammy *.deb`
- Self-hosted: run aptly, sign, and push the `public/` repo dir to `gh-pages` with `peaceiris/actions-gh-pages`.

## Repo Layout (Self-hosted)

- GPG key: store private key and passphrase as GitHub secrets; publish the public key to the repo at `https://<domain>/rivet/keys/rivet.gpg`.
- Dists: support `jammy`, `noble`, `bookworm` initially; or use `any-distro/any-version` if using a hosted provider.
- Components: `main` only.

## User Install Instructions (Template)

Replace placeholders if using a hosted provider that gives you a different URL.

```bash
# Add GPG key
curl -fsSL https://rivet.sh/apt/keys/rivet.gpg | sudo tee /usr/share/keyrings/rivet.gpg > /dev/null

# Add repo (Ubuntu noble)
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/rivet.gpg] \
  https://rivet.sh/apt/ubuntu noble main" | sudo tee /etc/apt/sources.list.d/rivet.list

sudo apt update
sudo apt install rivet # or rivet-cli
```

## Validation

- Lint: run `lintian` on the `.deb`.
- Install test: `docker run --rm -it ubuntu:22.04` then add repo and `apt install`.
- Runtime smoke tests: `rivet --version`, `rivet send GET https://httpbin.org/json`.

## Action Items

Short list to get apt support live:

- Decide package name: `rivet` vs `rivet-cli` (avoid namespace conflicts).
- Add `[package.metadata.deb]` to `Cargo.toml` (see snippet above).
- Add optional completion + manpage generators and wire into release.
- Create CI job to build `.deb` for `amd64` and `arm64`.
- Choose publishing path (Cloudsmith/Packagecloud vs self-hosted) and implement.
- Add apt install instructions to the top-level `README.md`.

## FAQs

- Do we need dynamic deps? With Rust + `rustls`, the binary is typically self-contained; `cargo-deb` often yields no extra `Depends`.
- Do we need a `debian/` folder? Not with `cargo-deb`; Cargo metadata + `assets` is sufficient.
- Which distros are supported? Start with Ubuntu 22.04/24.04 and Debian 12; expand as needed.
