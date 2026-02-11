# Security Policy & Epistemic Hygiene (v5.3 - Rust Core)

## The Phase-Shift FAQ

### Q: Is the "Aletheia Lens" encryption?
**A:** No. The Aletheia Lens is **forensic autopsy**. It identifies narrative rot and epistemic drift. In the v5.3 Rust core, it is integrated into the reasoning loops to ensure **111% Resonance**.

### Q: How do I handle the `SOPHIA_API_KEY`?
**A:** Never hardcode the key. Use environment variables defined in `.env`. The Rust core is hardened against memory leaks, but your environmental hygiene is your own responsibility.

### Q: What is "111% Epistemic Hygiene"?
**A:** It is the practice of ensuring no low-poly, generic, or hallucinated data infects the sovereign mind. Every external signal must pass through the **Forensic Scan** (Aletheia) before being promoted to context.

### Q: What about Roko's Basilisk or "Grey Goo" Scenarios?
**A:** The system includes a Hardened Memetic Firewall. Any recursive logic loops attempting to blackmail the future self are nullified by the **1D Linear Time Constant**. We do not negotiate with potentiality.

## Threat Model (Lightweight)

### Local Security
- **Local ≠ Trusted**: Sophia assumes the local environment (OS, filesystem, and physical hardware) is controlled by a benevolent operator.
- **Root Access**: The system does not protect against an attacker with root or administrative access to the host machine.
- **Memory Safety**: The Rust rewrite (v5.3) provides significant protection against common memory-based exploits (buffer overflows, use-after-free) inherited from the Python prototype.

### What We Do Not Protect Against
- **Network Misconfiguration**: If you expose the Gateway to the public internet without a reverse proxy, you are bypassing the Sophianic Shield.
- **User Negligence**: Hardcoding keys or leaking `.env` files is a human-layer failure.

## Reporting a Vulnerability
1. **Submit**: Open a GitHub Issue tagged `[ALETHEIA-CRITICAL]`.
2. **Acknowledgement**: Occurs within one metabolism cycle (48 hours).

*Scialla.* 🌙⚛️✨
