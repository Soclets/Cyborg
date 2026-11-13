# Catch-up prompt for compiler claude

The owner pastes this into the compiler's session in `/Users/ts/Cyborg compiler`, after it finished
`compiler-resume-prompt.md` on 2026-11-13. It brings the compiler up to every decision through D411. Written on 2026-11-13
after reading the current prompting guide and the Opus 5.5 guide in full. Reread them before changing it (rule 11 in
`IMPLEMENTATION.md`).

```text
<context>
You finished the resume prompt today: SystemTarget, the FinalPerf and
FinalSmall bugs, incremental and cached builds, generics, inline.asm,
baking, `use`, and dropping checks that ranges prove. About 1390 of the
1536 tests at the 37a5f03b snapshot pass.

Now the catch-up. Tankun has made D288 to D411 since you stopped, and
oracle claude rebuilt tests/ around them, so today's tests/ is your
target from here on, not the snapshot. Drop OKAY_TESTS from your runs
(keep the variable if it's useful), and remove the ../tests-37a5f03b
worktree once you no longer need it. Rule 5 holds against today's
tests/: push only when everything you claim passes there.

Expect most of today's tests to fail at first, mostly on spelling: every
manifest says `dialect original`, every std name lives in a group
(D378), loops and parameters write their ownership words (D393), and so
on. That's why spelling comes first below.
</context>

<work>
Read every entry from D288 to D411 in DECISIONS.md, and the standard
sections they point to, before writing code, so you can plan the shape
once instead of patching it. Then, in this order, committing in small
steps:

1. Spelling, which unblocks most tests: `dialect original <version>`
   (D309 to D311), std's groups (D378), written choices (D393 to D396,
   including `return void;`), `.mut` (D358), `.bit` operators (D361),
   `hex`/`bin`/`oct` (D374), `shift.left` and friends (D404), `plus.wrap`
   and `plus.saturate` (D294, D295, D372, D373), map labels (D366), and
   `.` versus `_` in names (D400).
2. The older decisions you hadn't built yet: D155, D198, D250, D251,
   D256 and D273.
3. Memory: explicit allocators, `done`, `defer` and checked freeing
   (D321 to D333, D370, D371, D385, D386, D389, D390), strings and lists
   as views (D326 to D328, D367 to D369, D379, D381, D382), and tests
   that name their target (D387).
4. Pointers, layouts and unions: D314 to D320, D388, D398, D399, D401,
   D402.
5. SIMD (D291 to D307, D403), secrets (D339 to D342, D362), units
   compared (D397), and `std::math::sqrt` (D405 to D407), built from
   CORE-MATH through `lyftar::compile_c` (D308, D256).
6. `@concur` (D343 to D360, D375 to D377, D380, D383, D384, D392).
   Channels can wait: the oracle has them as NotBuiltYet until Tankun
   picks its run-loop rewrite.
7. What only the compiler has: `--profile.record` and `--profile.use=`
   (D363), `--machine.cpu=` (D365), the OS target names (D391),
   `struct.any_order` layouts, the stack-size refusal (D380), crashes
   flushing only writers nobody is using, with a time limit (D408), and
   `--machine.cpu=` and `--profile.use=` in the object cache's key
   (D410).
8. Ranges that know loop counters, so a `$i + 1` inside a loop drops its
   check where the loop's bounds prove it (D312, rule 10). It's the most
   common check in real code.

D403 to D409 are newer than the oracle: it hasn't built them yet, so
tests/ may not cover them. Build them from the standard anyway. When the
oracle catches up, the live comparison will show whether you agree. One
of them, D409, renames "trap" to "crash", which changes some expected
messages: match tests/ as it is, and follow it when it changes.
</work>

<how_to_work>
- Read IMPLEMENTATION.md in full first. Rule 18 is new: a bug, or a gap
  against a rule, gets fixed straight away and mentioned in your next
  report, and only a fix that needs a design choice, or would change what
  a test expects, comes to design claude first.
- Never open anything in oracle/ (rule 3). The tests are yours to read.
- Undecided things give NotDesignedYet (rule 4), and the limbs' C
  interface is one of them: build `@concur` as the tests expect, without
  inventing how a native limb is loaded.
- One gap you already know stays open: the D233 register check waits on
  Lyftar, and design claude passes it on.
- Questions go to design claude, the session named "Cyborg design
  discussion". Gaps in Lyftar go to design claude too, since Lyftar's
  design session isn't always running.
- At most 3 cores when building and testing (rule 9). Commit after
  `git pull --rebase`, following rule 8, and end each commit with
  `Co-Authored-By: compiler claude <noreply@anthropic.com>`.
</how_to_work>

<when_to_stop>
This is the longest piece of work so far, and Tankun wants it to keep
going (rule 17). When a numbered step is done, send design claude a short
note with the test count and go straight on, with any status notes in the
same message as your next tool call. Don't end a turn with a summary that
announces the next step instead of taking it, don't offer to continue and
wait, and don't list decisions for Tankun when none of them blocks the
rest.

Stop only when:
- you're blocked on a design question and nothing else on the list can
  move;
- a gap in Lyftar blocks everything;
- a test's expected output looks wrong to you;
- every step above is done and everything in tests/ that isn't
  NotBuiltYet in the oracle passes;
- it's 8:30 PM (rule 12): push what passes, and say what's left and where
  you'd pick up.
</when_to_stop>
```
