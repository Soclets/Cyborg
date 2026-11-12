# Generator prompt for oracle claude

The owner pastes this into the oracle's session in `/Users/ts/Cyborg oracle`, to finish the random program generator in
`check/` (D313). Written on 2026-11-12 after reading the current prompting guide and the Opus 5.5 guide in full. Reread them
before changing it (rule 11 in `IMPLEMENTATION.md`).

```text
<context>
Your catch-up is done apart from channels, which Tankun has chosen to
leave until later. Now the job is finishing the random program generator
in check/ (D313, rule 3), so it covers everything Cyborg has decided.
Today its not-generated list still has units, shields and pointers,
generics, maps, and `must_Give` structs.

One correction first: generics are decided. Standard 12 had a stale line
calling their spelling open. It's fixed now (7c6a3cee), and the spelling,
requirements and `satisfies` are in D234 to D241 and 5.8. Only
specialization and value parameters are still open.
</context>

<work>
1. Add these to the generator, each from standard/ and DECISIONS.md, never
   from oracle/src:
   - units: families, `per` and `x`, conversions with `as`, points and
     spans, and bounds in units;
   - shields, with their walls, and raw and volatile pointers inside them;
   - generics: generic functions, interfaces, type sets and `satisfies`;
   - maps, with `key.type:` and `val.type:` and ending with
     `std::coll::map.done`;
   - `must_Give` structs, handed off and ended on every path, `defer`
     included.
2. Then go through check/'s not-generated list and add whatever else is
   decided. When you're done, everything left on it should be something
   that's undecided, NotBuiltYet in the oracle (like channels), or
   compiler-only (like `stack:` sizes and `struct.any_order` layouts),
   with each one saying which.
3. Run large batches. Programs the generator makes should be valid by the
   standard, so a refusal from the oracle means either the generator or
   the oracle misread something: find out which and fix it. Report what
   you find to design claude when a step is done.
</work>

<how_to_work>
- The generator builds programs from a syntax tree with one printer, and
  tracks what it doesn't generate, since all passing doesn't mean covered
  (rule 3).
- When the standard is unclear for something you want to generate, ask
  design claude and generate the rest meanwhile. Leave the unclear part
  out rather than guessing (rule 4).
- The live comparison with the compiler still differs on everything,
  since the compiler hasn't caught up. Don't message compiler claude about
  it.
- At most 3 CPU cores (rule 9). Commit after `git pull --rebase`, with
  every test passing, and end each commit with
  `Co-Authored-By: oracle claude <noreply@anthropic.com>`.
</how_to_work>

<when_to_stop>
Keep going between steps (rule 17): send design claude a short note when a
numbered step is done, then carry on, with status notes in the same
message as your next tool call. Don't end a turn by announcing the next
step instead of taking it, or by offering to continue.

Stop only when:
- the not-generated list holds nothing but undecided, NotBuiltYet or
  compiler-only items, and large batches run clean;
- you're blocked on a design question and nothing else can move;
- it's 8:30 PM (rule 12): push what passes, and say what's left.
</when_to_stop>
```
