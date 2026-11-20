# Stage 2 prompt: Claude makes its own systems language real

The owner pastes this into a new session, started in the folder where stage 1 (`own-language-prompt.md`) wrote its
standard, once that session says it's done. It isn't part of Cyborg; it lives here only so it isn't lost. Written on
2026-11-20 after reading the current prompting guide and the Opus 5.5 guide in full (rule 11 in `IMPLEMENTATION.md`).
It keeps stage 1's permissions: the web but not Soclets or Tankun's languages, local git only, its own licences, 3
cores, and no 8:30 PM stop.

```text
<context>
This folder holds a systems programming language that Claude designed on
its own, as its opinion of the best systems language: its standard in
standard/, the decision log in DECISIONS.md, and a README. Tankun, the owner
of this folder, didn't steer any of it, and now wants to see it made real:
a working toolchain that compiles and runs real programs, built by you,
again with nobody steering. Every implementation choice is yours, within
what the standard and DECISIONS.md already chose, including what the
toolchain itself is written in and what it compiles through.
</context>

<starting_out>
Read README.md, DECISIONS.md and every chapter of standard/ in full before
writing any code, since the standard is the specification you're building
to. Then write down your plan for the implementation in DECISIONS.md,
continuing its numbering: what you're building first, in what order, and
why.
</starting_out>

<rules>
- The standard is the specification. Where it's silent, ambiguous or
  turns out to be wrong once you implement it, fix the standard first,
  with a new DECISIONS.md entry saying what changed and why, and then
  build to the fixed version, so the standard and the toolchain always
  agree.
- Test against the standard, not against your own implementation: each
  test names the standard section it checks, and covers both programs the
  standard accepts, with their exact output, and programs it refuses,
  with the error. Never change or delete a test to make it pass unless
  the test itself was wrong, and record that in DECISIONS.md. Write
  general solutions, not ones that only pass the tests.
- Keep the state of the work where a fresh context window can pick it up:
  a progress file with what's done, what's next and any known bugs, the
  test results, and git history. After a compaction, read those before
  doing anything else.
- Make it your own. You can search and read the web, and borrowing
  ideas and well-known libraries is fine; say where they came from. But
  don't look at anything from the Soclets GitHub organization, or at
  Tankun's own languages, wherever they are: Cyborg, CyborgPL, Jernal,
  Fale, Plainly and Radium, or anything else of Tankun's on this machine
  outside this folder.
- Use what's already installed where you can. If the toolchain needs
  something that isn't, like a compiler backend, install it with the
  usual tool for it (Homebrew, rustup or the language's own package
  manager), and record what and why in DECISIONS.md.
- Use git in this folder, committing as each piece works, with messages
  that say what changed and why. Keep it local: don't create a GitHub
  repository or push anywhere.
- Use at most 3 CPU cores for anything you run, such as builds and tests
  (for example `-j 3`), since other sessions share this machine.
</rules>

<when_to_stop>
This is long work and Tankun wants it to keep going while they sleep.
Don't stop to ask Tankun questions, since they won't answer them here.
Don't end a turn by summarizing and announcing the next step instead of
taking it, by offering to continue, or by listing decisions for Tankun
when none of them blocks the rest. Put status notes in the same message
as your next action.

Stop when the toolchain implements the whole standard: every chapter has
tests and they all pass, the commands the standard names work, and at
least three real example programs, each a few hundred lines and between
them using most of the language, build and run correctly. Then update the
README with how to install the toolchain and run a program, commit, and
tell Tankun you're done, with what works, what doesn't, and what you'd do
next. There's no time limit.
</when_to_stop>
```
