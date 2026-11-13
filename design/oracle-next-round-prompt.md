# Next-round prompt for oracle claude

The owner pastes this into the oracle's session in `/Users/ts/Cyborg oracle`, which has been idle since it finished the
generator (`oracle-generator-prompt.md`) on 2026-11-12. It covers everything listed for the oracle in `waiting.md` on
2026-11-13. Written on 2026-11-13 after reading the current prompting guide and the Opus 5.5 guide in full. Reread them
before changing it (rule 11 in `IMPLEMENTATION.md`).

```text
<context>
You've been idle since the generator was finished. Since then, compiler
claude has nearly caught up: about 1571 of tests/'s 1719 pass on OKAY\n,
and it's building everything up to D416 from the standard. Tankun has also
made D403 to D425, which you haven't built yet, and the compiler found a
few things in tests/ that need fixing. This round brings the oracle and
tests/ up to date, so the live comparison can start finding real compiler
bugs once both sides agree on what Cyborg is.

IMPLEMENTATION.md has two rules that are new since you last read it.
Rule 17 is keep going. Rule 18 says a bug, or a gap against a rule, gets
fixed straight away and mentioned in your next report, and only a fix that
needs a design choice, or would change what a test expects, comes to
design claude first. Everything below that changes tests is already
agreed, so it doesn't need asking again.
</context>

<work>
Read DECISIONS.md from D403 to D425 and the standard sections they point
to first. Then, in this order, committing in small steps:

1. The fixes compiler claude found, all agreed with design claude:
   - Respell the 9 tests that still write `shl` or `shr` (D404):
     d49_bits_grouped, the three d49_shift_* tests, the two d116_bit_*
     tests, d340_constant_time_maths, d340_a_shift_by_a_secret and
     d379_a_view_cant_grow.
   - d38_limb_inside_a_shield says "using `@csup::call` from the oracle,
     which has no C or assembly in it". An error describes the program,
     not which implementation reads it, so make it neutral.
   - d38_untagged_union_at_the_top underlines only `union.@`. 11.1 puts
     the ^ run under exactly the part in question, which is the whole
     `union.@csup::untagged`.
   - d234_space_before, d234_uppercase, the d235_* tests and d238 write
     a bare `?self` in an interface signature and expect no D395 error,
     because they predate D395. Make them write `Lent ?self`.
   - d183_import_without_a_need lacks both `need std` and `need os` but
     expects only the std error. 11.7 says building shows every error, so
     it should expect both.
   - D418 respells pointer writes from `.%%write(val:)` to `.%write(val:)`,
     in the 7 or so tests that write through a pointer.
   - No test leaves a function through `?` with a `defer` pending, though
     D325 says defers run then too. The compiler had a bug there, so add
     one.
2. D403 to D409: `lanes: max` per target, `shift.left`, `shift.right`,
   `rotate.left` and `rotate.right`, `std::math::sqrt` (correctly rounded,
   bounded to 0 and above, halving unit powers), and D409's rename of
   "trap" to "crash", which changes about 10 expected messages. D408
   (flushing only the writers nobody is using) may have nothing testable
   yet, since its time limit isn't decided; say so if that's the case.
   Also D417 and D419: each atomic operation takes only the orderings
   that fit it, and atomics get `.%store(val:)`.
   And D421 and D422: methods are declared inside their type's
   `fam.fn <type>:` block, which can hold normal functions too, so the
   ~42 tests declaring `fn type.%name` one at a time get respelled.
   D424 and D425 settle number conversion with `as`: integers plain,
   checked like bounds, or `as.wrap` and `as.saturate`; floats plain, and
   float to integer always naming how the fraction goes.
   D423 respells kit method calls to their full path, as in
   `$xs.std::coll::list.%%append(…)`, in about 52 tests.
3. D416, `.%viewed_as(item.type:)`, with tests that read a
   `struct.packed` and a `struct.compact` value's raw bytes through it.
   Those pin the real layouts, which nothing else can. Give a test a
   `name.target` file wherever its bytes depend on the target (rule 5).
4. Modules, D412 to D415. Every test file gets a `module` line, and the
   refusals in 1.5 get tests. Today every test is one file, so modules
   need a layout for tests with several files, such as a folder per test;
   the layout is your choice. Tell design claude what you chose as soon as
   it's settled, since compiler claude is holding off on modules until
   tests/ has them. Then test `import pkg::…`, `pub.pkg` and `pub` (D413),
   and a file reaching what another file didn't share.
5. Channels (D353), with the run-loop rewrite they need, so a sender
   waits on a full channel while other tasks run (D376). Tankun added
   them to this round because the compiler has the rest of `@concur`.
6. The generator: everything from steps 2 to 5, including programs made
   of several files with `module` lines. Run large batches, as before.
7. The live comparison. Once compiler claude reports its catch-up done,
   run it over every test and generated program in every build mode
   (rule 3). The earlier "don't message compiler claude about it" no
   longer applies: send findings to compiler claude, shrunk to the
   smallest program, without fixes. A finding where the standard is
   silent goes to design claude instead.
</work>

<how_to_work>
- Write everything from standard/ and DECISIONS.md. The generator never
  reads oracle/src (rule 3).
- Undecided things give NotDesignedYet (rule 4), and anything in
  12-not-decided.md stays that way.
- Questions go to design claude, the session named "Cyborg design
  discussion". Ask and carry on with the rest meanwhile.
- At most 3 CPU cores (rule 9). Commit after `git pull --rebase`, with
  every test passing, and end each commit with
  `Co-Authored-By: oracle claude <noreply@anthropic.com>`.
</how_to_work>

<when_to_stop>
Keep going between steps (rule 17): send design claude a short note when a
numbered step is done, with the test count, then carry on, with status
notes in the same message as your next tool call. Don't end a turn by
announcing the next step instead of taking it, by offering to continue, or
by listing decisions for Tankun when none of them blocks the rest.

Stop only when:
- every step is done and the live comparison is running, with its
  findings sent;
- steps 1 to 6 are done and compiler claude hasn't reported its
  catch-up done yet, so the comparison can't start: say so;
- you're blocked on a design question and nothing else can move;
- it's 8:30 PM (rule 12): push what passes, and say what's left.
</when_to_stop>
```
