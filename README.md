# equuid

**The thoroughbred of UUID generators.** A UUID toolkit for the command line,
written in Rust.

Equuid is built around one idea: **a UUID is not an opaque string.** It carries a
version, a variant, and — depending on that version — a timestamp, a clock
sequence, a node, or a caller-defined payload. Most tools format those bits.
Equuid reads them, which is why dissecting a UUID sits alongside generating one
rather than behind it.

It aims at the whole of [RFC 9562][rfc] rather than the popular subset: every
version the RFC defines, every textual representation it defines, and the
compact radix encodings that fit 128 bits into 22 characters for the places a
UUID does not comfortably fit as text.

It also aims to be a **drop-in for the system `uuidgen`**. The same binary
answers to that name and accepts util-linux's option set, so it can take over
from the tool already on your machine without breaking the scripts that call it.

[rfc]: https://www.rfc-editor.org/rfc/rfc9562.html

> **Status: pre-alpha — design phase.** There is no code yet. The crate skeleton
> lands with the first code pull request, and nothing here is usable, stable, or
> installable. Watch the repository if you want to follow along.

## What it is for

- **Generating** UUIDs across the versions the RFC defines — the time-based
  ones, the name-based ones, the random one, and the escape hatch.
- **Dissecting** one: decoding a UUID you were handed into the fields it
  actually contains, rather than the string it happens to be wearing.
- **Validating** one in a script, with an exit code that distinguishes "this is
  malformed" from "you called me wrong".
- **Converting** between representations — the four the RFC defines, and the
  compact radix forms.
- **Emitting** machine-readable output, with the integer-precision problem
  handled rather than inherited.

## Design

Two properties shape most of the decisions, and they are the two most worth
knowing before reading the code:

- **The RFC's own vectors are the tests.** Appendix A and Appendix B are
  checked into the repository as data, and the implementation is asserted
  against them. The specification ships its own acceptance criteria; there is
  no reason to invent different ones.
- **Time and randomness are seams, not ambient state.** Generation is
  reproducible under a fixed clock and a seeded random source, which is what
  makes those vectors testable and keeps behavior stable across refactors.

The fuller design record is not published yet. It is being settled alongside
the implementation, and its shape is still moving — a record published now
would read as a commitment, and every later change would become a correction
rather than a decision. Decisions land in this repository with the code that
depends on them.

## Contributing

Read [`CONTRIBUTING.md`](CONTRIBUTING.md) first. The short version: keep the
design invariants intact, one pull request per concern, and commits are
certified under the [DCO](CONTRIBUTING.md#sign-off-your-commits-dco). There is
no CLA.

## License

`MIT OR Apache-2.0`, at your option — see [`LICENSE-MIT`](LICENSE-MIT) and
[`LICENSE-APACHE`](LICENSE-APACHE). Contributions come in under the same terms
(`inbound = outbound`).

The equuid name, and any logo, **as brand identifiers** are governed separately
— see [`TRADEMARK.md`](TRADEMARK.md). There is no logo file yet; when one lands
it will carry the same license as the code.

Equuid is a project of the [Infinite Primates](https://github.com/infiniteprimates)
org. The copyright holder is named in [`AUTHORS.md`](AUTHORS.md).
