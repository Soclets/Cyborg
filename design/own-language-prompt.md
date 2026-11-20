# Kickoff prompt: Claude designs the systems language it thinks is best

The owner pastes this into a new session, started in an empty folder, to see what Claude Opus 5.5 thinks the best
systems programming language is, designed with nobody steering. This is stage 1, the standard; stage 2, making it real,
is `own-language-build-prompt.md`. It isn't part of Cyborg; it lives here only so it isn't lost.

It lets the session make its own public GitHub repository under Soclets, choose its own licences, and use at most 3 CPU
cores, and it has no 8:30 PM stop, since it may work overnight (owner, 2026-11-17). The first version was pasted into
the "Systems programming language design" session on 2026-11-17, whose first reply was stopped by a safety classifier for
an unknown reason, and that session was abandoned with its folder empty. This version, rewritten on 2026-11-20 after
reading the current prompting guide and the Opus 5.5 guide in full (rule 11 in `IMPLEMENTATION.md`), asks for Claude's
opinion of the best systems language, rather than any language of its own.

```text
<context>
Tankun, the owner of this folder, wants to know what you think the best
systems programming language would be, and to see it designed with nobody
steering. Tankun designs several languages with Claude, but in this one
they won't weigh in on anything. Every decision is yours: what the language
is for, what it values, its name, syntax, type system, memory model, error
handling, tooling, and what it would eventually be built with. Aim for the
language you'd actually argue is the best design for systems programming,
not a safe compromise or a tour of features, and be honest in the log
about the trade-offs it loses.

This first stage is the standard only: a written specification, precise
enough that someone else could implement the language from it without asking
you anything. A later stage will build it, from this standard alone, so
anything the standard leaves vague will have to be guessed then.
</context>

<rules>
- Make it your own. Don't open or search anything outside this folder,
  especially Tankun's other languages (Cyborg, CyborgPL, Jernal, Fale,
  Plainly, Radium) or any of Tankun's other projects. Borrowing ideas from
  well-known languages is fine; say where an idea came from.
- Start by writing down what the language is for, what it values, and why
  you think those are the right values for a systems language. Then let
  every later decision follow from that.
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
- Use at most 3 CPU cores for anything you run, since other sessions share
  this machine.
</rules>

<when_to_stop>
This is long work and Tankun wants it to keep going while they sleep.
Don't stop to ask Tankun questions, since they won't answer design
questions here. Don't end a turn by summarizing and announcing the next
step instead of taking it, by offering to continue, or by listing
decisions for Tankun when none of them blocks the rest. Put status notes
in the same message as your next action.

Stop when the standard covers the whole language: syntax, types, memory,
errors, modules and packages, the standard library's core, and the
toolchain's commands, with every chapter complete enough to implement
from. Then write a short README saying what the language is, why you
think it's the best design for systems programming, and how the standard
is organized, push, and tell Tankun you're done. There's no time limit.
</when_to_stop>
```
