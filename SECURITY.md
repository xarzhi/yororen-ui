# Security Policy

Thank you for helping keep Yororen UI and its users safe.

## Supported Versions

Only the latest minor release line currently receives security updates. Because Yororen UI is pre-1.0 and moving quickly, we do not back-port fixes to older minor versions.

| Version | Supported          |
| ------- | ------------------ |
| 0.3.x   | :white_check_mark: |
| < 0.3   | :x:                |

## Reporting a Vulnerability

Please do **not** open a public issue for security vulnerabilities.

Instead, report security issues privately via [GitHub Security Advisories](https://github.com/MeowLynxSea/yororen-ui/security/advisories) so we can coordinate a fix and disclosure timeline before details become public.

When reporting, please include:

- A description of the vulnerability and its impact
- Steps to reproduce, or a minimal proof of concept
- The affected crate(s) and version(s) (e.g., `yororen-ui-core`, `yororen-ui-xml`, etc.)
- Your preferred disclosure timeline, if any

We aim to acknowledge reports within 5 business days and will keep you informed as we investigate and prepare a fix.

## Disclosure Policy

Once a fix is available, we will:

1. Publish a security advisory on GitHub
2. Release a patched version on crates.io
3. Credit the reporter unless they wish to remain anonymous

## Scope

Security issues we care about include, but are not limited to:

- Memory-safety issues in unsafe code paths
- XML macro injection or unsafe evaluation
- Theme / asset parsing vulnerabilities
- Anything in Yororen UI code that could compromise a host application

Issues in upstream dependencies (e.g., `gpui-ce`) should generally be reported to their respective maintainers, though we are happy to help coordinate if the issue affects Yororen UI usage.

## Security-Related Configuration

- Yororen UI does not execute network requests or run external processes
- The XML DSL is intentionally expression-less: it maps to Rust builder calls and does not evaluate arbitrary code
- Embedded assets are bundled at compile time via `rust-embed`
