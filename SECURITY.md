# Security Policy

## 🛡️ Commitment
We take the security of `cntm-graph` seriously. This project aims for a **High (Mission Critical)** security standard.

## 📢 Reporting a Vulnerability
Please do not report security vulnerabilities through public GitHub issues. Instead, send a detailed report to:
**security@cntm-labs.com**

### What to include:
- A description of the vulnerability.
- Steps to reproduce (PoC).
- Potential impact.

## 🔐 Security Protocols
All memory-mapped graph mutations must be atomic and verified against the formal logic defined in the Lean kernel to prevent structural corruption and ensure mission-critical reliability.

- **Dependency Management:** Regularly scan for vulnerable packages.
- **CI/CD Security:** Mandatory automated security scans are integrated into `.github/workflows/security.yml`.
- **Disclosure:** We follow a responsible disclosure timeline.
