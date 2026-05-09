# Security Policy

## Supported Versions

Currently, only the latest version of Knowledge Loom is supported with security updates.

## Reporting a Vulnerability

If you discover a security vulnerability, please report it privately.

### How to Report

1. **Do not** create a public GitHub issue
2. Send an email to the maintainers (contact information to be added)
3. Include as much detail as possible:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Any suggested fixes

### Response Timeline

- **Initial response**: Within 48 hours
- **Assessment**: Within 1 week
- **Fix and release**: As soon as feasible, typically within 2-4 weeks depending on severity

### What to Expect

- We will acknowledge receipt of your report
- We will work with you to understand and validate the issue
- We will coordinate a fix and release schedule
- We will credit you in the release notes (if you wish)

## Security Best Practices

Knowledge Loom is designed with security in mind:

- **Local-first**: All data processing happens on your machine
- **No cloud dependencies**: No data is sent to external services
- **Minimal permissions**: Only accesses files in your knowledge base
- **No network access**: Does not make outbound network requests (except for optional web server features)

### Recommended Practices

1. Keep your knowledge base backed up
2. Use file system permissions to restrict access to your knowledge base
3. Be cautious when enabling the web server feature
4. Review the code before running in production environments

## Security Audits

This project has not yet undergone a formal security audit. We welcome security researchers to review the code and report any issues found.

## Dependency Security

We use automated tools to monitor dependencies for known vulnerabilities:

- `cargo-audit` checks for security advisories in Rust dependencies
- `cargo-deny` enforces license and security policies

These checks run automatically in our CI/CD pipeline on every pull request.

## Disclosure Policy

We follow responsible disclosure practices:

1. Vulnerabilities are fixed before public disclosure
2. Users are notified of security updates
3. Security advisories are published with detailed information
4. Credit is given to reporters (with permission)

## License

This security policy is provided as-is without any warranties. For more information, see the project license (MIT OR Apache-2.0).
