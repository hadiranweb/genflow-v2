# Security Policy

## Supported Versions

| Version | Supported |
| --- | --- |
| 0.1.x | ✅ |
| 2.0.x | ✅ |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it via:

1. GitHub Security Advisories
2. Email: security@hadiranweb.com

Please do NOT disclose security issues publicly until a fix is available.

## Security Best Practices

- Never commit secrets or API keys
- Use environment variables for sensitive data (see `.env.example`)
- Keep dependencies updated (`cargo update` + `cargo audit`)
- Follow principle of least privilege
- JWT secrets must be changed in production (never use the default)
- Representative influence policy only affects Work Style axis (never hard requirements)
- Docker containers run as non-root user (`appuser`, uid 1000)
