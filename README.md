# equuid

**The thoroughbred of UUID generators.** A UUID toolkit for the command line,
written in Rust.

Covers [RFC 9562][rfc] in full. Drops in for util-linux's `uuidgen` 2.43.

[rfc]: https://www.rfc-editor.org/rfc/rfc9562.html

## What this is

A UUID looks like a string. It isn't one.

Every UUID carries a version, a variant, and — depending on which version — a
timestamp, a clock sequence, a node, or a payload you supplied yourself. The
hyphens and the hex are just how those bits get written down.

Most tools stop at the string. Equuid reads what's underneath: ask it for a UUID
and it'll make you one, hand it a UUID and it'll tell you what's actually in
there. It does that for every version the RFC defines rather than the handful
everyone uses — and for all four of the ways it defines for writing one down,
plus the compact radix encodings that squeeze 128 bits into 22 characters for
the places text doesn't fit.

It's a drop-in replacement too, not just an alternative. The same binary answers
to `uuidgen` and takes the options util-linux's does, so the scripts you already
have keep working.

> **Status: pre-alpha, design phase.** There's no code yet. The crate skeleton
> lands with the first code pull request. Nothing here is usable, stable, or
> installable — watch the repository if you want to follow along.

## What you'll be able to do

- **Generate** UUIDs of every version the RFC defines — the time-based ones, the
  name-based ones, the random one, and the escape hatch for everything else.
- **Dissect** one you were handed, into the fields it actually contains rather
  than the string it happens to be wearing.
- **Validate** one in a script, with an exit code that tells "this is malformed"
  apart from "you called me wrong".
- **Convert** between representations — the four the RFC defines, and the
  compact radix forms.
- **Pipe** machine-readable output, with the integer-precision problem handled
  instead of inherited.

## How it's built

Two things drive most of the decisions here, and they're the two worth knowing
before you read the code.

**The RFC's own vectors are the tests.** Appendix A and Appendix B are checked
into the repository as data and asserted against the implementation. The
specification ships its own acceptance criteria; there's no reason to invent
different ones.

**Time and randomness are seams, not ambient state.** Generation is reproducible
under a fixed clock and a seeded random source, which is what makes those
vectors testable and keeps behavior from drifting when someone refactors.

The fuller design record isn't published yet. It's being settled alongside the
implementation and it's still moving — publish it now and it reads as a promise,
and every later change becomes a correction instead of a decision. Decisions
land here with the code that depends on them.

## Contributing

Start with [`CONTRIBUTING.md`](CONTRIBUTING.md). The short version: keep the
design invariants intact, one pull request per concern, and certify your commits
under the [DCO](CONTRIBUTING.md#sign-off-your-commits-dco). There's no CLA.

## License

`MIT OR Apache-2.0`, your pick — see [`LICENSE-MIT`](LICENSE-MIT) and
[`LICENSE-APACHE`](LICENSE-APACHE). Contributions come in under the same terms
(`inbound = outbound`).

One file sits outside that: [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) is adapted
from the Contributor Covenant and stays under CC BY-SA 4.0.

The equuid name, and any logo, **as brand identifiers** are governed separately
— see [`TRADEMARK.md`](TRADEMARK.md). There's no logo file yet; when one lands
it'll carry the same license as the code.

Equuid is a project of the [Infinite Primates](https://github.com/infiniteprimates)
org. The copyright holder is named in [`AUTHORS.md`](AUTHORS.md).
