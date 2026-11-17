# Kickoff prompt: Claude designs a systems language on its own

The owner pastes this into a new session, started in an empty folder, to see what Claude Opus 5.5 designs with nobody
steering. It isn't part of Cyborg; it lives here only so it isn't lost. Written on 2026-11-17 by design claude, after
reading the current prompting guide and the Opus 5.5 guide in full (rule 11 in `IMPLEMENTATION.md`).

It lets the session make its own public GitHub repository under Soclets, choose its own licences, and use at most 3 CPU
cores (owner, 2026-11-17). Unlike Tankun's own sessions, it has no 8:30 PM stop and may work overnight (owner,
2026-11-17).

```text
<context>
Tankun, the owner of this folder, wants to see what you design when nobody
steers you: a systems programming language, made entirely by your own choices.
Tankun designs several languages with Claude, but in this one they won't
weigh in on anything. Every decision is yours: the language's goals, its
name, syntax, type system, memory model, error handling, tooling, and what
it would eventually be built with.

This first stage is the standard only: a written specification, precise
enough that someone else could implement the language from it without asking
you anything. No compiler or interpreter yet.
</context>

<rules>
- Make it your own. Don't open or search anything outside this folder,
  especially Tankun's other languages (Cyborg, CyborgPL, Jernal, Fale,
  Plainly, Radium) or any of Tankun's other projects. Borrowing ideas from
  well-known languages is fine; just say where an idea came from.
- Start by writing down what the language is for and what it values, then
  let every later decision follow from that.
- Keep a decision log, DECISIONS.md, numbered and never renumbered. Each
  entry says what was chosen, what alternatives you weighed, and why this
  one won. A changed decision gets a new entry saying which one it
  replaces. Tankun reads this to understand your thinking, so the "why"
  matters more than anything else in it.
- Write the standard in standard/, as numbered chapters, with a final
  chapter listing what's still undecided. Each rule shows a short example
  and says which decisions it comes from.
- When something is genuinely open, pick an answer and record it, rather
  than leaving it vague. A standard full of "TBD" can't be implemented.
- Use git in this folder, committing as you finish each piece, with
  messages that say what changed and why. Once the language has a name,
  create a public GitHub repository for it under the Soclets organization
  with `gh`, named after the language, and push as you go.
- Choose the language's licences yourself, for its code and for its
  writing, record why in DECISIONS.md, and add the licence files before
  the first push, so the repository is never public without one.
- Use at most 3 CPU cores for anything you run, such as builds or tests
  (for example `-j 3`), since other sessions share this machine.
</rules>

<when_to_stop>
This is long work and Tankun wants it to keep going. Don't stop to ask
Tankun questions, since they won't answer design questions here. Don't end a
turn by summarizing and announcing the next step instead of taking it, or
by offering to continue. Put status notes in the same message as your next
action.

Stop when the standard covers the whole language: syntax, types, memory,
errors, modules and packages, the standard library's core, and the
toolchain's commands. Every chapter should be complete enough to implement
from. Then write a short README saying what the language is and how its
standard is organized, and tell Tankun you're done. There's no time limit:
Tankun will be asleep while you work, so keep going through the night if
the standard needs it.
</when_to_stop>
```
