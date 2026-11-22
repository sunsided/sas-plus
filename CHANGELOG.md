# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2](https://github.com/sunsided/sas-plus/compare/v0.2.1...v0.2.2) - 2026-11-22

### Changed

- Link the crate to its page on the planning and search site ([#5](https://github.com/sunsided/sas-plus/pull/5))
- Link the crate to its page on the planning and search site
- Report dependencies and workflow findings to GitHub code security
- Describe the audience and the PDDL bridge in the README
- Document the tests and clear their pedantic findings
- Document every item and split the long functions in the library
- Enable pedantic clippy, doc lints and forbid unsafe code
- Name release PRs imperatively and skip Actions bumps in the changelog
- Write a contributing guide
- Set up git hooks and Copilot review instructions
- Extend the README with badges, install, MSRV and license sections
- Render mermaid on docs.rs and doctest the README example
- Cover more invariants with proptest and a projection fuzz target
- Move the integration tests into one tests/it binary
- Declare licenses per file with REUSE
- Check dependencies with cargo-deny, Dependabot and typos
- Configure Codecov for the coverage upload
- Mirror CI in the Taskfile and split it into named jobs
- Stop tracking Cargo.lock

## [0.2.1](https://github.com/sunsided/sas-plus/compare/v0.2.0...v0.2.1) - 2026-11-19

### Changed

- Check the no_std builds and the MSRV before publishing
- Pin the release tag format and drop the Dependabot changelog rule
- Automate releases with release-plz and trusted publishing
- Saturate distance sums and accept the full range in Rng::range
- Clarify the AxiomLayer error and the SasAxiom pre rule
