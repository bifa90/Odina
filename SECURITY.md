# Security policy — Odina

Odina is an antivirus. A vulnerability in it is not a normal bug: it runs with
the user's privileges, it reads files it did not create, and it will eventually
receive updates. Please report privately.

## Reporting a vulnerability in Odina itself

**Do not open a public issue.** Use GitHub's private advisory form:

<https://github.com/bifa90/Odina/security/advisories/new>

Include:

- what is affected (engine, CLI, installer, update path, website),
- the exact version or commit,
- reproduction steps, and the smallest file that shows the problem,
- what an attacker gains.

Do **not** attach a live malware sample. A SHA-256 and a description are
enough; we will arrange a private channel if the file is needed.

## What to expect

| Step | Target |
|------|--------|
| Acknowledgement | 7 days |
| Initial assessment | 14 days |
| Fix or mitigation plan | 30 days for high severity |
| Public credit | after the fix ships, if you want it |

This is a one-person project. The targets are honest intentions, not a
contract, and they are published so a silence is visible rather than assumed.

## Scope

In scope:

- `odina-core`, `odina-cli`, the installer and the update channel;
- the signing and release pipeline;
- the website deploy script and the fonts manifest verification.

Out of scope:

- a false positive on a file you believe is clean — that is a **quality** bug,
  please open a normal issue with the SHA-256 and no sample;
- a missed detection — also a normal issue, but read `CONTRIBUTING.md` first;
- attacks that require the attacker to already control the user's account or
  the machine with the same privileges Odina runs at.

## Safe harbour

Good-faith research on your own devices, or devices you are authorised to test,
is welcome. Do not test against third parties, do not upload samples to this
repository, and do not use the project to attack anyone.

## No bounty

There is no money for reports. What exists is a fix, a credit, and a project
that treats the report as a security report rather than a support ticket.
