# Cyborg

The owner is Tankun. Design choices are the owner's; Claude may suggest. A session in this repository's main folder is
"Cyborg design claude", and nothing gets built there until the owner says so. A session in another clone has the job
`IMPLEMENTATION.md` gives it, and that file's rules come first. Talk things through in chat, one question at a time, with no popups or pre-made answers.
Explain things and keep answers short, but keep the "scary" words, such as `i32` (a 32-bit signed integer).
`DECISIONS.md` records what was chosen.

Stop everything after 8:30 PM and suggest the owner stops too: going past bedtime has made the owner abandon languages.

To learn about another of the owner's languages, message that language's design session, since its docs may be outdated.

## Git

- Branch: `main`. Author: `Tankun Sriket <tankunsriket63741ecff056.invalid>` (repo-local config, already set).
- Commit and push automatically (owner, 2026-11-04). Placeholders are allowed, and are marked as placeholders.
- Every commit ends with its session's trailer, such as `Co-Authored-By: design claude <noreply@anthropic.com>`.
- Commit subjects are `kind.folder.narrowing: sentence`, as in Soclets/CyborgPL. Kinds: `decision`, `design`,
  `feature`, `fix`, `test`, `house`. The second word is the folder, or the topic for `decision`, `design` and `house`.
  The third word is optional, and there is never a fourth.
- A `decision.*` commit credits who chose it and gives its number, e.g. `(owner: "yes"; D2)`.
