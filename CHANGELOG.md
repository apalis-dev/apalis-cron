# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Add dependabot changelog

### Added
- CI/CD pipelines with GitHub Actions
- Automated publishing to crates.io
- Documentation deployment to GitHub Pages
- Security auditing with cargo-audit
- Dependency management with Dependabot
- Code coverage reporting
- Multi-platform testing (Linux, macOS, Windows)
- Feature matrix testing
- Improve cargo vet

### Changed

### Deprecated

### Removed

### Fixed

### Security

- Added cargo audit and vet

## [1.0.0-beta.2] - 2025-11-16

### Added
- Initial release of apalis-cron
- Cron-based scheduling with standard cron expressions
- Timezone support for job scheduling
- Builder pattern for schedule creation
- English-to-cron translation support
- Serde serialization support
- Integration with apalis ecosystem
- Persistence support for distributed cron jobs
- Extensible middleware support

[Unreleased]: https://github.com/apalis-dev/apalis-cron/compare/v1.0.0-beta.2...HEAD
[1.0.0-alpha.1]: https://github.com/apalis-dev/apalis-cron/releases/tag/v1.0.0-beta.2
