# Contributing

Thanks for contributing to **equuid**. This is a small, focused project, so the
bar is simple: keep the design invariants intact, and certify that you have the
right to submit what you send.

## The invariants

These are the properties the design exists to hold. A change that breaks one
needs a strong argument, and the argument belongs in the pull request.

1. **RFC 9562 is the specification.** The published vectors in Appendix A and
   Appendix B are the acceptance tests, and they are checked in as data before
   the code that satisfies them. A change that breaks a vector is wrong; a
   vector is never "updated to match" an implementation.
2. **Determinism is enforced, not hoped for.** No wall clock and no OS randomness
   is read anywhere except inside the `Clock` and `Rng` implementations.
   Everything downstream of those two traits is a pure function of its inputs,
   which is what makes the vectors reproducible.
3. **The dependency set is deliberate.** Every dependency is a maintenance and
   supply-chain cost. `serde`/`serde_json`, `bs58` and `rand` were each declined
   with reasons; the design record has them. Adding a dependency needs an
   argument in the pull request, not just a passing build.
4. **The legacy `uuidgen` surface is frozen; the ergonomic surface is ours.**
   They do not share a parser. The legacy option set is fixed by an external
   tool's history, so coupling the two would mean every future ergonomic change
   risks breaking compatibility.

## Getting started

The crate skeleton lands with the first code PR — see issue **#1** (`cargo init`
with a lib/bin split). The toolchain is pinned in the repository, so the version
you build with is the version CI builds with; install nothing by hand.

From then on the local gate is:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

CI runs the same three steps, plus the MSRV job. A change is not ready for
review until that MSRV job is green — the floor is a tested claim, not a
documented hope.

## Sign off your commits (DCO)

There is **no CLA** — see [`AUTHORS.md`](AUTHORS.md#contributors). Contributions
are accepted under the [Developer Certificate of Origin
1.1](https://developercertificate.org/), the same `inbound = outbound`
certification the Linux kernel uses: you certify that you had the right to
submit what you send.

Certify by adding a `Signed-off-by` line to every commit. Git does it for you:

```bash
git commit -s -m "parse: reject misplaced hyphens in the hyphenated form"
```

which appends, from your configured `user.name` and `user.email`:

```text
Signed-off-by: Your Name <you@example.com>
```

Use your real name and a reachable address. The sign-off is a first-person
statement about a contribution you made — do not sign off on someone else's
behalf. (The one narrow exception is the maintainer certifying an agent-authored
branch; see [Remediation](#remediation).)

A pull request fails the DCO check if any commit in it is missing a valid
sign-off. [Remediation](#remediation) is how that gets fixed without rewriting
history.

The full text you are certifying:

```text
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.


Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

## Automation

Part of this project is written by AI agents (the infiniteprimates agent team)
working under the maintainer's authority. They are tooling, not authors — see
[`AUTHORS.md`](AUTHORS.md#automation). That has two consequences for how commits
are made:

- **Agents never add `Signed-off-by`.** The sign-off is a first-person legal
  certification and a non-person cannot make one. The DCO certifies a
  contributor's right to submit the work; agents are tooling, they hold no
  copyright, and the maintainer is the only party with rights in the change — so
  an agent commit contains nothing for anyone to certify. DCO2 acts on exactly
  that: it reports `Skipped: sign-off not required in bot commit` and passes a
  bot-only pull request. [Remediation](#remediation) below is the fallback if
  that ever changes.
- **Agent commits carry an `Assisted-by:` trailer** naming the agent, instead:

  ```text
  Assisted-by: <login>[bot] <<id>+<login>[bot]@users.noreply.github.com>
  ```

  For example:

  ```text
  Assisted-by: zeroclaw-tony[bot] <329726292+zeroclaw-tony[bot]@users.noreply.github.com>
  ```

  `<id>` is the bot account's numeric id, which GitHub embeds in the commit
  address. It changes if the account is renamed or replaced, so read it from the
  App rather than from this file.

  A trailer discloses; it does not certify. Keep it to one line: GitHub's
  squash-message builder generates a `Co-authored-by:` line for every distinct
  author on a branch, so a branch that mixes identities republishes all of them
  into `main`'s history.

## Remediation

A **remediation commit** retroactively adds a missing sign-off. It is a new
commit, so history is not rewritten and no one's work is disturbed. Both forms
are enabled in [`.github/dco.yml`](.github/dco.yml).

It is a fallback rather than a routine step: DCO2 skips bot-authored commits, so
an agent-authored branch normally passes the check without one.

### Individual

Authored by the same person as the commits it covers:

```text
DCO remediation commit for Your Name <you@example.com>

I, Your Name <you@example.com>, hereby add my Signed-off-by to this commit: <SHA>
I, Your Name <you@example.com>, hereby add my Signed-off-by to this commit: <SHA>

Signed-off-by: Your Name <you@example.com>
```

### Third-party

Authored by the maintainer on behalf of the failing commit's author, certifying
a contribution they hold the authority to certify:

```text
Third-party DCO remediation commit for <author>

On behalf of <author>, I, <maintainer>, hereby add my Signed-off-by to this commit: <SHA>

Signed-off-by: <maintainer>
```

Only sign off for someone whose authority you actually hold — you cannot certify
a contributor you have no relationship with.

A remediation commit moves the pull request's head, and the ruleset dismisses
stale reviews on push. Approve **after** it lands, not before.

> A failed DCO check normally also offers a **Set DCO to pass** override button.
> It is disabled in this repo — see [`allowOverrideAction`](.github/dco.yml). A
> remediation commit is the only path, because it records the certification in
> git history while the override would record it only in GitHub's audit log. A
> maintainer can re-enable the button for a one-off by setting that key to
> `true`.

## Submitting changes

- Open an issue first for anything bigger than a small fix. The maintainers can
  tell you whether it conflicts with the invariants above — that conversation is
  cheaper before the code than after it.
- **One pull request, one concern.** Aim for roughly 400 lines of *code* diff,
  tests included; generated and vendored-verbatim files do not count against it.
  A prose document is reviewed as a document and splits by document, not by
  line. A PR that does two things gets reviewed as neither.
- Anything touching the parsing or generation paths, or a radix codec, is
  reviewed by the architect as well as a second reviewer; see [Review](#review).
- **The test corpus is data, not code.** New vectors are added to the checked-in
  table; they are not inlined into a test function, where they become invisible
  to a count.

## Review

Every pull request is reviewed by someone other than its author, and the
reviewer follows what the change touches:

| The change touches | Reviewed by |
| --- | --- |
| Infrastructure, CI, packaging | the infra owner |
| Tests, the vector corpus, benchmarks | the test owner |
| Parsing, generation, the radix codecs | the architect, always |

A review of a correctness change is expected to ask two things: which invariant
it holds, and which test would fail if it broke.

## Style

- Rust only, formatted with `rustfmt`, clean under `clippy -D warnings`.
- Comments explain *why*, not *what*. Where a decision was contested, cite the
  design-record section it came from.
- New behavior needs a test that fails without it. Where the property is itself
  the thing being tested — a round-trip, or monotonicity — prefer a property
  test to an example-based one.
- **Accept exactly what the RFC defines and reject near-misses.** A misplaced
  hyphen, a non-`uuid` urn namespace, a non-canonical radix encoding: these are
  errors, not input to be normalised. Silently accepting them hides the caller's
  bug until the data is somewhere it matters.

## Maintainers

Maintained by the [Infinite Primates](https://github.com/infiniteprimates) org,
on behalf of the copyright holder named in [`AUTHORS.md`](AUTHORS.md).
