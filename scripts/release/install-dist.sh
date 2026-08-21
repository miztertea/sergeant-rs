#!/usr/bin/env bash
# Bootstrap `dist` (cargo-dist) from its attested, checksum-pinned prebuilt
# tarball — NOT `curl … cargo-dist-installer.sh | sh`. Measured against
# axodotdev's own release assets (w3-recon.md §1, 2026-08-21):
# cargo-dist-installer.sh has no `.sha256` sidecar, is absent from the
# aggregate `sha256.sum`, and has no build-provenance attestation at all
# (`gh attestation verify cargo-dist-installer.sh --repo axodotdev/cargo-dist`
# → HTTP 404). The per-target tarballs have both.
#
# Two checks, deliberately, because they prove different things: the
# committed sha256 pins the exact bytes we qualified for this version (an
# attestation alone would accept a *different* axodotdev-built tarball served
# from the same URL), and `gh attestation verify` proves those bytes came out
# of axodotdev/cargo-dist's own release workflow rather than merely matching a
# hash someone uploaded. Neither check subsumes the other.
#
# Bumping the version means re-recording both sha256 literals below from that
# release's `sha256.sum` — deliberately, so a version bump is a qualification,
# not a one-character edit. Shared by both bootstrap call sites in
# .github/workflows/release.yml (package, package-installer) so there is one
# copy of the pinned hashes to re-qualify, not two.
#
# Requires GH_TOKEN in the environment (for `gh attestation verify`) and the
# GitHub Actions runner environment: RUNNER_OS, RUNNER_ARCH, RUNNER_TEMP,
# GITHUB_PATH.
#
# Usage: install-dist.sh <dist-version>
#   e.g. install-dist.sh 0.32.0
set -euo pipefail

dist_version=${1:?usage: install-dist.sh <dist-version>}

case "${RUNNER_OS}-${RUNNER_ARCH}" in
  Linux-X64)
    dist_target="x86_64-unknown-linux-gnu"
    dist_sha256="eb52f9fae0d0506774e9f1801c1168f87fa2c87a45e2d64d3ae7c89401929946"
    ;;
  macOS-ARM64)
    dist_target="aarch64-apple-darwin"
    dist_sha256="aa343b2ff78ec2981f17a65140250c5ad6062c74072163f68c5c2686d94763a7"
    ;;
  *)
    echo "::error::no qualified cargo-dist ${dist_version} tarball is pinned for runner ${RUNNER_OS}-${RUNNER_ARCH}" >&2
    exit 1
    ;;
esac

archive="cargo-dist-${dist_target}.tar.xz"
curl --proto '=https' --tlsv1.2 -LsSf --retry 3 -o "${RUNNER_TEMP}/${archive}" \
  "https://github.com/axodotdev/cargo-dist/releases/download/v${dist_version}/${archive}"
echo "${dist_sha256}  ${RUNNER_TEMP}/${archive}" > "${RUNNER_TEMP}/dist.sha256"
if command -v sha256sum >/dev/null 2>&1; then
  sha256sum -c "${RUNNER_TEMP}/dist.sha256"
else
  shasum -a 256 -c "${RUNNER_TEMP}/dist.sha256"
fi
gh attestation verify "${RUNNER_TEMP}/${archive}" --repo axodotdev/cargo-dist
mkdir -p "${RUNNER_TEMP}/dist-bin"
tar xf "${RUNNER_TEMP}/${archive}" --strip-components 1 -C "${RUNNER_TEMP}/dist-bin"
got=$("${RUNNER_TEMP}/dist-bin/dist" --version)
if [ "$got" != "cargo-dist ${dist_version}" ]; then
  echo "::error::extracted dist reports '$got', expected 'cargo-dist ${dist_version}'" >&2
  exit 1
fi
echo "${RUNNER_TEMP}/dist-bin" >> "$GITHUB_PATH"
