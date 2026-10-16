# Security Policy

## Supported versions

**None yet.** equuid is pre-alpha: there is no released version, no published
artifact, and nothing to install. Only the tip of the default branch is looked
at.

This section will list supported versions once there is a release to support.

## Reporting a vulnerability

Please **do not** open a public issue for security-sensitive findings.

Report vulnerabilities privately to the maintainers via GitHub's
[private vulnerability reporting](https://docs.github.com/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability)
feature on this repository, or to the maintainer contact published by the
[Infinite Primates](https://github.com/infiniteprimates) org.

Please include:

- A description of the issue and its impact
- Steps to reproduce, or a proof-of-concept if available
- Affected versions, or the commit you tested

## Scope

In scope: the code in this repository — the parsers, the generators, the radix
codecs, the format detection rule, the JSON writer, and the CLI's argument
handling.

Two things worth stating plainly:

- **A UUID is an identifier, not a secret.** v4 draws its 122 random bits from
  the operating system source, so the value is unpredictable — but that is not
  the same as being a key or a token, and nothing here makes it one. Do not
  reach for this tool when you need a secret; reach for a secret.
- **The tool parses untrusted input.** A panic, an unbounded allocation, or an
  index-out-of-range reachable from a malformed UUID or a malformed radix string
  is a security-relevant finding. The parsers and the decoders are the attack
  surface, and the radix decoders are the sharpest edge of it — they accept
  attacker-chosen alphabets and lengths.

There is no upstream project to redirect you to. equuid is written from scratch
and vendors and redistributes no third-party code; if that changes, this section
changes with it.

## What to expect

- An acknowledgment within a reasonable window.
- A good-faith effort to triage and, for in-scope issues, publish a fix.
- This is a small project with no SLAs. If you need a guaranteed response
  window, maintain a hardened internal fork.

## Threat model

Not written yet. It lands with the first code — there is no threat model worth
publishing before there is a system to model. Until then, treat the invariants
in [`CONTRIBUTING.md`](CONTRIBUTING.md) as design intent rather than as
assurance.
