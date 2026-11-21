---
applyTo: "**/Cargo.toml,release-plz.toml,.github/workflows/release-plz.yml,.github/workflows/publish-crates-io.yml,**/CHANGELOG.md"
---

# Crates and releases

- Versions are SemVer 2.0 (Cargo's interpretation: in `0.x`, a minor bump is breaking).
  Pre-releases (`-rc.1`) are fine, but build metadata (`+...`) is not.
- Versions and `CHANGELOG.md` are managed by release-plz release PRs. Do not bump versions
  or edit generated changelog entries by hand in feature PRs.
- `sas-plus` is published and other planning crates depend on it. Flag any public API
  change; `cargo-semver-checks` (CI job `semver`, and release-plz) compares against the
  release on crates.io.
- The crate has `description`, `license`, `repository`, `readme`, `keywords` (at most 5) and
  `categories` (valid crates.io slugs), and declares `rust-version`. The fuzz crate has
  `publish = false`.
- The single crate `sas-plus` is tagged `vX.Y.Z` (`git_tag_name` in `release-plz.toml`).
- New dependencies must pass `cargo deny check`: allowed licenses, no advisories, no git or
  unknown-registry sources. `serde` is the only (optional) runtime dependency; adding one
  needs a reason in the PR.
- `rust-version` is a signal, not a contract. Raising it is allowed when a dependency or a
  feature benefits. It needs a minor release and a changelog note, never a patch release.
- release-plz runs in git-only mode: it tags and creates GitHub Releases.
  `publish-crates-io.yml` publishes from the tag in the `crates-io` environment, via trusted
  publishing (`id-token: write`, no `CARGO_REGISTRY_TOKEN`). Keep it that way, and do not
  rename the workflow file or the environment.
- The package excludes `tests/fixtures/translated/`, `tests/translated.rs`, the GPL license
  text and the repository tooling. Check `cargo package --list` when that list changes.
- Changelog groups and minor bumps come from imperative subject verbs (`release-plz.toml`).
  A subject that hides a feature behind "Update ..." lands under Changed. Flag it.
