# Cyborg — design decisions

The owner decides; design claude proposes and records. Each entry says what was chosen and whose idea it was.
Numbers are permanent, so a decision is never renumbered. A changed decision gets a new entry that says which one it
replaces.

## Decided

**D1. Where Cyborg comes from.** Cyborg takes inspiration from Radium and Rust, and from the owner's taste. Too much
consistency is stressful, a lesson from around 10 abandoned languages, so consistency is not a goal in itself.
(Owner, 2026-11-04)

**D2. Radium's ownership words, unchanged.** As Radium's design session described them on 2026-11-04:

| At the call | On the parameter | Meaning |
|---|---|---|
| `Give $x` | `Given` | Ownership moves, and `$x` can't be used afterwards |
| `Copy $x` | `Given` | The function owns a separate copy, and the caller's is untouched |
| `Lend $x` | (bare) | Temporary read-only access, and the caller keeps it |
| `Lend.mut $x` | `Lent.mut` | Temporary access that may change it, and nothing else uses it meanwhile |

The call and the parameter are both written, and they must match. A bare parameter is lent; writing `Lent` alone is
refused. Integers and `bool` are always copied and take no word. A type marked `must_Give` is linear: it is handed off
with `Give` exactly once, or the program does not compile. Stored borrows are types: `Lend from $r reading`,
`Lend from <$a, $b> reading`, `Lend from program reading`, `Lend.mut from …`. No word is needed to read in place
(`$xs.len`, `for $x in $xs`) or for a fresh value that has no holder yet. Claude asked which words to change; the owner
chose: "no change". (Owner, 2026-11-04)

**D3. No implicit ownership.** Every hand-off shows on the page, including one to a method's receiver (the thing before
the dot). This rules out `change $xs = $xs.append(val: 4)` with a silently given receiver, and Claude's proposal that
`change` stand in for `Lend.mut`. The owner: "No implicit Ownership". (Owner, 2026-11-04)

**D4. `%` marks an unsaid ownership.** A `%` on a method's name says the receiver's ownership is unsaid here and is
found in the signature: `$xs.%append(val: 4)`. The compiler still checks it, and a search for `%` finds every unsaid
hand-off. The owner's idea. (Owner, 2026-11-04)

**D5. What `%` may hide.** `%` goes only on a method call, never on an argument, so `total(%$xs)` is refused. It hides
only `Lend` and `Lend.mut`, so after a `%` the value is always still alive. Giving away is spelled out:
`close(Give $f)`. Claude proposed it; the owner chose it: "your recom". (Owner, 2026-11-04)

**D6. Implementation waits for Lyftar.** Lyftar is the owner's in-house LLVM toolchain. Cyborg stays in design until
Lyftar is ready. (Owner, 2026-11-04)

**D7. `as` converts.** `$t as str` builds a new value of another type, which can allocate memory or fail. `:` no longer
converts. Claude first suggested `as`, then dropped the push; the owner chose it after reading `if $n: bool:` as a type
test: "maybe we'll go `as` for the conversion anyways". (Owner, 2026-11-04)

**D8. Methods alongside narrowings.** Cyborg has both. A narrowing makes the thing before the `.` more specific
(`fn.main`, `std::list.i32`), and a method is a function attached to a type, called on a receiver
(`$xs.%append(val: 4)`). A `.` never runs code and a `.%` always does. Since D5 keeps `Give` out of `%`, methods only
borrow, and anything that consumes its value is a plain function. Claude asked whether to drop methods and use
`append(Lend.mut $xs, val: 4)`; the owner chose both: "yea. Because, CyborgPL was genuinely too long". (Owner, 2026-11-04)

**D9. `if $s: status.failed:` tests one case.** Cyborg has both `match`, which must cover every case, and this test,
which reads as "if `$s` is the failed kind, do this". Inside the block the compiler treats `$s` as that case (flow
typing), so `$s.code` is safe there and a compile error outside. Adding a case to a union later forces every `match` to
handle it, while this test keeps compiling and skips it. The owner read `if $n: bool:` as a type test; Claude proposed
applying it to union cases; the owner chose it: "Ah, yes, yes". How unions are declared is not settled here.
(Owner, 2026-11-04)

**D10. `i32`, and no chain.** A 32-bit signed integer is spelled `i32`, and `int.signed.32_bit` does not exist, so there
is one spelling and no alias. Claude proposed it; the owner chose it: "almost every sys programmer knows `i32`, and has
a stroke on `int.signed.32_bit`. So, keep". (Owner, 2026-11-04)

**D11. Nothing goes unsaid about being empty or unbounded.** A list always writes its `max_len`, with `inf` for
unbounded, an empty list always writes `= empty`, and `main` says `return void;`. Claude proposed a bare `std::list.i32`
meaning unbounded; the owner kept it written out, for the vibe: "it's consistent, and I don't really have a problem with
it". Whether every `void` function says `return void;` is not settled here. (Owner, 2026-11-04)

**D12. Both `array` and `list`.** Cyborg has both, and `list` keeps its name. What separates them and how `array` is
spelled are not settled here. The owner: "I actually meant to have `array` and `list`, but I forgot to include in
example". (Owner, 2026-11-04)

**D13. Labels can have narrowings.** A label is the name in `val: 4` or `len: 8`, and it can be narrowed where a type's
writer defines that. Bare `len` is exact, `len.max` is a limit and `len.min` is a floor, so `std::array.i32[len: 8]` is
exactly 8, `std::list.i32[len.max: inf]` is any number, and `std::list.i32[len.min: 1, len.max: 8]` is 1 to 8. A list
with a bare `len` is refused in favour of `array`. There is no `len.always`. This replaces the word `max_len` in D11, and
settles what D12 left open. The owner proposed narrowed labels with `len.always` and `len.max`; Claude proposed bare
`len` and `len.min`; the owner chose: "Heck yes. And no `.always`". (Owner, 2026-11-04)

**D14. Five build modes.** From the owner's sketch, written up by a friend's Claude:

| Mode | Optimization | Runtime checks | Notes |
|---|---|---|---|
| `Working` | you choose | all kept, no matter the cost | for developing and debugging |
| `FinalSafe` | max | cheap checks only | the default, even when benchmarking, and noted |
| `FinalPerf` | max | none | trust your tests |
| `FinalShit` | max | none, more UB possible than C | comes with a curse easter egg |
| `FinalSmall` | for size | whatever makes it smaller | UB and bad speed allowed, size only |

The same sketch says: anything the Precog limb proves can drop its check safely, in any mode; contracts are checked at
runtime unless Precog proves them; and linter strictness, such as whether `mut` must actually change, is separate from
the build mode. Claude pointed out that with no checks, a failed check in `FinalPerf` becomes UB; the owner accepts that:
"you test your shit throughly first". What Precog and contracts are in Cyborg is not settled here. (Owner, 2026-11-04)

**D15. The curse in `FinalShit`.** Building in `FinalShit` prints this, from the owner:

```
$ cyborg build --mode FinalShit

warning: you chose FinalShit
what:  every contract is now a fucking promise nobody checked
why:   you asked for more UB than C, you absolute legend
fix:   use FinalSafe, or pray

Compiling... ☠️
Done. Good fucking luck.
```

(Owner, 2026-11-04)

**D16. Radium's error format, plus warnings.** As Radium's design session described it on 2026-11-04:

```
refused: main.cy:12:5, line 12 column 5
12 |     close(Give $f);
   |     ^^^^^^^^^^^^^^
 what: `$f` was already given away
 why:  a given value is dead afterwards (D2)
 because: main.cy:9:5, line 9 column 5
 9 |     send(Give $f);
   |     ^^^^^^^^^^^^^
     `$f` was given away here
```

The first word is the kind: `refused:` (a definite mistake while building), `unproven:` (while building, the prover could
prove it neither way), `check failed:` (while running), and `warning:`, which Cyborg adds. The place is written twice:
`file:line:col` for tools, with the column in UTF-8 bytes, then `line N column M` for people, with the column in grapheme
clusters. Then comes the code line, with a `^` run under exactly the part in question. `what:` and `why:` are always
present, `why:` cites the decision number, and there are no error codes. `when:` gives the values that break it, where
known. `because:` links show earlier steps that led to the error, only when the compiler knows they did. Building and
running share one layout. A message says only what is certain: with exactly one possible fix, `fix:` replaces `what:` and
shows it as code; with more than one, there are no suggestions. The `FinalShit` curse is exempt. The `.cy` extension in
the example is a placeholder. Claude proposed taking it whole; the owner chose it: "yup". (Owner, 2026-11-04)

**D17. Another line in the curse.** This replaces the text in D15. The owner asked for the extra line "somewhere", and
Claude placed it after `fix:`. The spelling is the owner's:

```
$ cyborg build --mode FinalShit

warning: you chose FinalShit
what:  every contract is now a fucking promise nobody checked
why:   you asked for more UB than C, you absolute legend
fix:   use FinalSafe, or pray
your shit better not be commited

Compiling... ☠️
Done. Good fucking luck.
```

(Owner, 2026-11-04)

**D18. The curse's extra line varies.** The line from D17 is spelled `your shit better not be committed`, correcting D17,
and it is one of several variations. The other lines are not chosen yet. The owner: "the correct one, and that line
should have some variations". (Owner, 2026-11-04)

**D19. The curse's lines.** The extra line from D18 is one of these. Claude proposed all but the first and last; the
owner chose: "Every, and one more", which is the last:

```
your shit better not be committed
this better not be on main
git blame will find you
your future self already hates you
tell your coworkers you're sorry
this isn't going to prod, right?
nobody is coming to save this binary
hope you like debugging with no stack trace
if this ships, you never saw me
Even Grok wouldn't fucking joke about this
```

Claude suggested a random pick on each build, which changes only the printed message and never the binary; the owner did
not object. (Owner, 2026-11-04)

**D20. A plain `=` checks bounds.** `let $b: i32[min: 0, max: 100] = $n;`, `change $b = $b + 1;` and `use_score($n)` all
compile, and each does a runtime check when the compiler can't see the value fits. An out-of-range value crashes, or is
UB in a build mode with no checks (D14). A literal that can't fit, like `150` here, is a compile error, arithmetic gives
back a plain `i32`, and a bounded value goes into a plain `i32` with no check. `if $n: i32[min: 0, max: 100]:` is how to
handle a bad value yourself, and it stays in every build mode. Claude argued for `as` until the `+ 1` example, then for
this; the owner chose it: "we could just go b". (Owner, 2026-11-05)

**D21. A linter, eventually.** Cyborg gets a tool like Rust's Clippy, separate from the compiler and the build modes
(D14). It can force comments, and it is how the hidden checks in D20 can be made visible. Its name and rules are not
settled here. The owner: "How about eventually a tool that forces comments? And other stuff like Clippy for our lang".
(Owner, 2026-11-05)

**D22. `＠` marks the limb part, and only that.** Bare `std::` reaches the kit part of `std`, a library, and `＠std::` reaches
the limb part, a compiler extension: `＠std::inline.asm`. `＠` goes only on things a limb provides, so `＠std::println` is
refused, and a search for `＠` finds every compiler extension in the code. It is separate from the `%` in D4. The owner's
idea, told to Radium's design session: "`std::` bare means the kit part only (lib), with `＠` it means the limb part too";
Claude asked whether "too" meant both parts; the owner chose: "limb only". (Owner, 2026-11-05)

**D23. Radium's bracket rule.** `( )` runs something, `[ ]` describes or holds values, and `< >` fills in a type or a
source. So a function is declared with `fn total [$xs: …] -> i32:` and called with `total(Lend $xs)`. The one exception
is grouping in maths, `($a + $b) x 2`. An opening `<` has no space after it, a closing `>` has none before it, and a
comparison has a space on each side, so `$a<$b` is refused. Claude first suggested `( )` for declaring, then `[ ]` after
reading the rule; the owner chose: "that rule". (Owner, 2026-11-05)

**D24. A collection's types go in `< >`.** `std::list<i32>[len.max: inf]` and `std::array<i32>[len: 8]` replace the dot
spellings in D8 and D13. A map's types get labels, such as `std::map<key: str, val: i32>`. The label names are Claude's
placeholder. Claude proposed `< >`; the owner chose it and added the labels: "`<>`, and btw `std::map<str, i32>`. That
shit will very much get labels, of course". (Owner, 2026-11-05)

**D25. When `< >` needs labels.** One type can go bare, and more than one must have labels: `std::list<i32>` is fine,
`std::map<key: str, val: i32>` is required, and `std::map<str, i32>` is refused. Claude proposed it; the owner chose it:
"that rule". (Owner, 2026-11-05)

**D26. No default build mode.** You always say which mode you're building in, as in `cyborg build --mode FinalShit`. This
replaces "the default" in D14's `FinalSafe` row. UB stays possible in `FinalPerf`, `FinalShit` and `FinalSmall`; in
`FinalSafe`, cheap checks such as overflow, bounds and division by zero stay as traps. Claude asked whether UB stays with
`FinalSafe` as the safe default; the owner chose: "yes, but there's no default build mode". (Owner, 2026-11-05)

**D27. What Cyborg is.** A general systems language, started from scratch, with ideas merged rather than code. From the
owner's chat with Radium's design session: "Not literally merging, more like merging ideas, and starting from scratch.
And maybe tune down big changes, that's what made CyborgPL insane to maintain. It's gonna be a general sys lang, and it's
long because there's no assumptions anymore." (Owner, 2026-11-05, relayed)

**D28. `std`'s two parts.** The kit part is written in Cyborg and built by the compiler, and the limb part lives in the
compiler, so it's in Rust, with its names and signatures visible in Cyborg. `inline.asm` comes from `std`: "`inline.ASM`
comes from the std kit & limb itself", later lowercase. Radium's design session proposed the split and the owner replied
"yea", which it says may have answered its whole explanation. "And like Radium-style stuff could be a limb". (Owner,
2026-11-05, relayed)

**D29. Precog and contracts.** Precog's layers 1 and 2, known values and ranges, are in the core. Layer 3, Z3 and CVC5, is
a limb. That limb adds invariants, `assert` hints, ghost code, lemmas, proofs that loops end and that stack and heap fit
their budgets, proof levels, and Radium's "prove everything or refuse" mode as an option. Contracts are in the base, which
the owner overruled Radium's design session on: "contracts go in base". The rest it proposed, and the owner chose it:
"yes". (Owner, 2026-11-05, relayed)

**D30. The linter is an official limb.** The tool in D21 ships with the compiler and is configured in the project file.
Definite mistakes stay compiler errors. Radium's design session proposed it; the owner chose it: "yes". (Owner,
2026-11-05, relayed)

**D31. C and C++.** CyborgPL's reach: C headers read directly, C and C++ layouts, calls through C's conventions, and no
deep C++. Cyborg adds generated `.h` and `.hpp` files for C++ calling in, and walls at the boundary. Radium's design
session proposed it; the owner chose it: "yes". (Owner, 2026-11-05, relayed)

**D32. Published benchmarks use `FinalSafe`.** A published Cyborg benchmark is built in `FinalSafe` and says so. This keeps
what D14 said about benchmarks, now that D26 has removed the default. Claude proposed keeping it; the owner chose: "yes".
(Owner, 2026-11-05)

**D33. `union.tagged` says it.** A tagged union is declared `union.tagged status:` and closed with
`end union.tagged status`, and there is no bare `union`, because "union" means opposite things to a C programmer and a
Rust one. Its cases are narrowings, such as `status.failed`, and a case's values go in `[ ]`, such as `failed[code: i32]`.
This settles the declaration D9 left open. Claude proposed a bare `union` meaning tagged; the owner chose otherwise:
"accuracy really matters for this one. Why don't just say it?" (Owner, 2026-11-05)

**D34. Shields.** Cyborg has Radium's shields: a named block with a written reason, the only place dangerous code may
live, listed with its size in the build report. Cyborg isn't all proven, so a shield here is closer to Rust's `unsafe`
unless the Precog limb's strict mode is on. Claude proposed bringing them in; the owner chose it: "yes". (Owner,
2026-11-05)

**D35. C support is a limb.** The limb `@csup` holds all of D31: C headers, C and C++ layouts, C calls, and generated
headers. C's union is `@csup::union.untagged number:`, inside a shield. The base language has no C in it, and a project
lists the limb when it needs it. `＠std::inline.asm` stays in `std`. The owner asked why the union had no mark and suggested
`@csup`; Claude proposed that it hold all of D31; the owner chose it: "yes". (Owner, 2026-11-05)

**D36. The limb mark is a plain `@`.** `@std::inline.asm` and `@csup::union.untagged`. This replaces the fullwidth `＠`
written in D22 and D35, which Claude copied from Radium without asking. Claude proposed it; the owner chose: "This one:
@". (Owner, 2026-11-05)

**D37. A shield's exit checks follow the build mode.** They stay in `Working`, stay in `FinalSafe` even when expensive, are
gone in `FinalPerf` and `FinalShit`, and are gone in `FinalSmall` if that makes it smaller. This differs from Radium,
where they stay in every mode. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-05)

**D38. What must live in a shield.** `@std::inline.asm`, raw pointers, `@csup` C calls and `@csup::union.untagged`. Using a
limb doesn't need a shield by itself: `@` says it's a compiler extension, and a shield says it's dangerous. Claude
proposed it; the owner chose: "yea". (Owner, 2026-11-05)

**D39. A shield's other walls.** Entry: `requires` is checked at runtime unless Precog proves it, following the build mode.
Containment: raw pointers and unchecked borrows can't leave a shield, as a compile error, in every mode except `FinalShit`,
which goes full UB. Claude proposed the walls and keeping containment even in `FinalShit`; the owner chose: "yea. But
`FinalShit` is comedic, so it would just go full UB". (Owner, 2026-11-05)

**D40. `FinalShit` assumes nothing bad can happen.** It drops every compile-time safety rule, such as ownership, borrows,
`must_Give`, containment, what must live in a shield and bounds on literals, so a program can build only in `FinalShit`.
The compiler still refuses what it can't compile: bad syntax, unknown names and types that don't fit. Claude proposed
dropping the rules; the owner put it: "Oh, it assumes nothing bad can happen". Claude reads that as the optimizer also
treating contracts and bounds as facts, which is what would make it "more UB than C" (D14); the owner hasn't confirmed
that reading. (Owner, 2026-11-05)

**D41. `FinalPerf` assumes too, seriously.** `FinalPerf` treats contracts and bounds as facts for the optimizer, because it
assumes you've tested thoroughly. It isn't comedic: it keeps every compile-time safety rule and has no curse, which are
what still separate it from `FinalShit`. Claude suggested keeping assuming to `FinalShit`; the owner chose otherwise:
"`FinalPerf` isn't comedic, it assumes you have tested it throughly already. But it shouldn't be comedic". (Owner,
2026-11-05)

**D42. When a plain `+` overflows.**

| Mode | What happens |
|---|---|
| `Working`, `FinalSafe` | Trap |
| `FinalPerf` | Wraps |
| `FinalShit` | UB, because it assumes nothing bad can happen |
| `FinalSmall` | Whatever's smaller |

`FinalPerf` assumes the promises you wrote (D41), but a plain `+` isn't one. A wrapping operator, spelled `plus.wrap` as a
placeholder, wraps in every mode. Claude proposed UB in `FinalPerf`; the owner asked "Shouldn't FinalPerf wrap instead of
UB?", and Claude agreed and proposed this table; the owner chose it: "yes". (Owner, 2026-11-05)

**D43. The operators with symbols.** `+`, `-`, `x`, `!=`, `=`, `==`, `?` and `??`. Multiply is `x`, as in `$a x $b`. Claude
proposed `*` and `/`; the owner chose: "My taste is: `+`, `-`, `x`, `!=`, `=`, `==`, `?`, `??`". Claude reads anything not
listed, such as divide and `<`, as a word; the owner hasn't confirmed that. What `?` and `??` do is not settled here.
(Owner, 2026-11-05)

**D44. `?`, `??`, `<` and `>`.** `?` means `try`: if this failed, hand the failure to the caller. `??` means `otherwise`: if
this failed, use this instead. `<` and `>` join the symbols in D43, with D23's spacing. The owner: "`?` = `try`, `??` =
`otherwise`. `<`, `>` is also my taste". Whether the words `try` and `otherwise` also exist is not settled here.
(Owner, 2026-11-05)

**D45. Symbols only, and an `otherwise:` block.** `?` and `??` have no word forms, so there is no `try`. `otherwise:` exists
as a block, for handling a failure in more than one line. Handler collections can be used here, and how is not settled.
Claude proposed symbols only; the owner chose: "symbols only, and `otherwise:` could be a block, and btw you could use
handler collections". (Owner, 2026-11-05)

**D46. Handlers, and `*` for the ones that can jump.** A handler narrowing is a named set of `otherwise` arms, used at a call
as if written there: `otherwise word_failed.use_placeholder;`. The arms must cover every failure still possible at that
call. A handler that can `return`, `break` or `continue` is marked `*` where it's used, `otherwise word_failed.*bail;`, and
the compiler requires the mark on one that can jump and refuses it on one that can't. The spelling `narrowings.handler` is
Radium's, shown here without objection. The `*` comes from CyborgPL; Claude proposed it; the owner chose it: "yes".
(Owner, 2026-11-05)

**D47. Failures.** A function that can fail says so on its return type, `-> file or fail[file_failed, bad_syntax]`, mixing
named sets with one-off failures. A named set is a collection, and a failure can carry data in `[ ]`:

```
narrowings.failure file_failed:
    file_failed.not_found;
    file_failed.denied;
    file_failed.disk_full[free: i32];
end narrowings.failure file_failed
```

`?` is a compile error if the function doesn't list what it passes up. Two sets sharing a failure's name need the full
`file_failed.denied` in the list. `or fail[…]` comes from CyborgPL. The owner proposed reusable failures and asked "Should
it have a collection for that?"; Claude proposed `narrowings.fail`; the owner chose: "yea, but `narrowings.failure`". The
statement that raises a failure is not settled here. (Owner, 2026-11-05)

**D48. No bare `div`, and lots of options.** Division has no single right answer for integers, so there is no bare `div`: you
pick one, such as `div.trunc`, `div.floor` or `div.exact`. More generally, Cyborg offers many options through narrowings,
such as `str.utf16`. Claude proposed a truncating bare `div`; the owner chose: "just let them fucking pick, then. My ass.
Cyborg should have a lot of options, fucking `str.uft16` and shit". The full list of `div` narrowings and what `div.exact`
does in each mode are not settled here. (Owner, 2026-11-05)

**D49. The word operators.** `mod`, `at_least`, `at_most`, `and`, `or`, `not`, `bit_and`, `bit_or`, `bit_xor`, `bit_not`, `shl`
and `shr`. Claude proposed them; the owner objected only to the bare `div` (D48) and raised nothing against the rest. (Owner,
2026-11-05)

**D50. A bare name is the obvious default.** A bare `str` means UTF-8, and `std::println` isn't written
`std::println.shortest.stdout`. Where no default is obvious there is no bare name, as with `div` (D48). Claude asked whether
to always pick an encoding; the owner chose: "look at `std::prinln`. It's not `std::println.shortest.stdout`". (Owner,
2026-11-05)

**D51. Spelled-out defaults.** A default the language or `std` fixes can't be written out, so `str.utf8` and
`std::println.stdout` are refused, with the fix shown. In a collection you write yourself, it's allowed, since your default
might change. This is Radium's split. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-05)

**D52. `fails` goes in the body.** A function lists its failures as the first thing in its body, and raises one with `fail`:

```
fn half [$n: i32] -> i32:
    fails [odd, negative[value: i32]];
    if $n mod 2 != 0:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half
```

This replaces `or fail[…]` on the return type in D47; the rest of D47 stands, so the list can mix named sets and one-offs.
The owner asked "Maybe in the body instead?"; Claude offered declaring it first in the body or not at all; the owner chose:
"A". (Owner, 2026-11-05)

**D53. Contracts go first in the body.** In a fixed order: `requires`, `ensures`, `fails`, then the code. So the top of a
function's body says what it promises, and the signature stays short:

```
fn half [$n: i32] -> i32:
    requires $n at_least 0;
    ensures result at_least 0;
    fails [odd];

    if $n mod 2 != 0:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half
```

The spelling, including `result`, is Claude's. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-05)

**D54. `f32`, and likely `d32`.** A 32-bit binary float is `f32`. A decimal float is likely `d32`. Claude suggested
`f32.decimal`; the owner chose: "yea, but decimal is likely just gonna be `d32`, because `f32.decimal` looks cursed". Whether
`d32` is in the base, or marked as a limb or a kit, is not settled here. (Owner, 2026-11-05)

**D55. `d32` is a limb.** Decimal floats are provided by the compiler and marked `@`, not base and not a kit. Whether the limb
is `std` or its own is not settled here. Claude proposed `@std::d32`; the owner chose: "a limb, but I don't know about it
being in `std`". (Owner, 2026-11-05)

**D56. Bare `@` is a compiler builtin.** With `::`, as in `@csup::union.untagged`, `@` is a limb. Without it, as in
`@size_of<i32>()`, it's a builtin, always available. The test for a builtin: it looks like a function call, and it can't be
written as a Cyborg function. So `let`, `+` and `as` aren't builtins, and neither is `std::println`, and `@let` can't happen.
The owner asked whether bare `@` should mean what Zig's does, and warned "it might become `@let`"; Claude proposed the test,
then suggested dropping bare `@`; the owner answered: "`::` already fucking says it's a limb when there's `@` infront". Claude
reads that as keeping both. There is no `@import`: kits are reached by their path. (Owner, 2026-11-05)

**D57. Structs.**

```
struct reading:
    celsius: i32;
    time: i64;
end struct reading

let $r: reading = reading[celsius = 21, time = 3600];
std::println($r.celsius as str);
```

Fields have no `$`. Building one uses `[ ]`, with `=` setting a field, and reading a field is a plain `.`. A linear type is
`struct.must_Give open_file:`. Claude proposed it; the owner chose: "Yes". (Owner, 2026-11-05)

**D58. `:=` means a constant.** `$b: i32 := 60 x 60;` has no `let`, and its value must be fully known at compile time, so
`$a: i32 := read();` is refused. It takes the maths meaning of "is defined as". The owner: "was thinking of letting `:=` means
const"; Claude proposed the compile-time rule; the owner chose it: "yea". (Owner, 2026-11-05)

**D59. `~` marks an inexact literal.** A float literal the type can't hold exactly must say so: `let $b: f64 = ~0.1;`. An exact
one has no mark: `let $a: f64 = 0.5;`, and `0.1` needs none in `d32`. Leaving it off is refused, with the fix shown, and writing
it on an exact literal is refused too. A negative is `~-0.1`. So a long constant can be pasted as `~3.14159265358979323846`,
and there is no limit on digits. `~` covers literals only, not arithmetic, and it joins the symbols in D43. Claude proposed
refusing more than 17 digits for `f64`; the owner proposed this: "you have to write something like `~0.1` when inexact? And no
`~` when exact"; Claude agreed; the owner chose it: "yea". Refusing `~` on an exact literal is Claude's addition. (Owner,
2026-11-05)

**D60. Rust host, LLVM backend.** The compiler is written in Rust, with no plan to self-host, though the `std` kit part is in
Cyborg (D28). The only backend is LLVM, through Lyftar. A Cranelift backend for `Working` could be added later without changing
the language, if slow builds hurt. Claude proposed it; the owner chose: "yes". (Owner, 2026-11-05)

**D61. No C++ in this repository.** The first implementation rule: if Cyborg needs something from LLVM that Lyftar can't do, it
gets added to Lyftar, including custom passes' support. The owner asked whether C++ would be needed here for custom LLVM passes;
Claude proposed the rule; the owner chose: "yea". (Owner, 2026-11-05)

**D62. A small oracle, the one exception to D6.** Cyborg gets an oracle in Rust, which needs no LLVM: only what's needed to run
and refuse real example programs, with no optimizing and no build modes. It's made in its own session, so this one stays on
design, and this session can stop it at the 8:30 PM limit. Claude proposed the oracle; the owner chose: "yea, but maybe made by
another session. So I will have time to talk to you about design, and you could stop anything when it goes past 8 something".
(Owner, 2026-11-05)

**D63. The rules hook.** Implementation rules live in a short `IMPLEMENTATION.md`, and a `SessionStart` hook in
`.claude/settings.json` prints it into a session at startup and after compaction, also trying `resume` and `clear`, which TAKL
didn't test. It delivers text and enforces nothing. From TAKL's `decision.house.rules_hook`; the owner called it the "hooker".
Claude proposed taking it; the owner chose: "yea". It isn't installed yet. (Owner, 2026-11-05)

**D64. The implementation rules.** `IMPLEMENTATION.md` holds them, adapted from TAKL's: three sessions, with oracle claude building
the oracle rather than design claude; twelve rules; tracing and naming. Rule 11 is the owner's change: "messaging other sessions
by yourself (AI) don't count as prompting, and don't write it like prompting either". Rule 10 is from the owner's experience: "I
learnt that from reimplementing Lua in SPARK". Rule 12 is Claude's addition, from the owner's bedtime rule. Claude proposed them;
the owner chose: "All of those". (Owner, 2026-11-05)

**D65. Lints.** Every crate here forbids `unsafe_code`, denies `clippy::unwrap_used`, `clippy::expect_used` and `clippy::panic`, and
warns on `clippy::pedantic`. This enforces rule 6 of `IMPLEMENTATION.md`, though an array index out of bounds can still panic. The
owner asked "Is it pedantic?" and noted Rust is less strict than SPARK; Claude proposed this; the owner chose: "yea". (Owner,
2026-11-05)

**D66. Verus on the important parts.** The compiler's important parts are proven with Verus. The oracle stays plain. Which parts
count is not settled here. Claude suggested leaving Verus out for now; the owner chose: "the important parts, yk? And I could let
Lyftar use Verus". Whether Lyftar does is for Lyftar's design session. (Owner, 2026-11-05)

**D67. What Verus proves.** The checks where a compiler bug would silently make users' code unsafe: ownership (`Give`, `Lend`,
`must_Give`), bounds (the checks in D20 and the test in D9), failure coverage (`match`, `otherwise`, `fails`), shield containment
(D38, D39), and the `@` and `%` rules (D5, D22). Parsing, error messages and generating LLVM IR aren't on the list; the oracle
comparison covers those. Claude proposed it; the owner chose: "yeah". (Owner, 2026-11-05)

**D68. check claude, later.** A fourth session, in `check/`, attacks the compiler as a separate author: it generates random valid
programs, shrinks failures, and compares the oracle and the compiler as black boxes. It reads neither's source, reports findings
without sending fixes, and starts once the compiler exists, though its generator can start earlier. oracle claude stays a separate
session, because a generator written by the oracle's author would share its misreadings. From CyborgPL's check claude, which
warned about syntax churn and untracked coverage. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-05)

**D69. Tools are Rust too.** No scripting languages in the repository. Tools go in a `tools/` crate, run with `cargo run`, and get
the same lints. There are two exceptions: one-line commands in config, like the hook's `cat`, and temporary scratch work that is
never committed. CyborgPL had 59 scripts by the end. The owner asked whether to allow scripting languages; Claude proposed this;
the owner chose: "yea". (Owner, 2026-11-05)

**D70. Scratchpad and downloads.** Scratch space is for what can be lost: anything needed later goes in the repository and gets
committed. And needing a download is not a cost: a dependency or tool is chosen on maturity, safety, maintenance and speed, so a
less mature, less safe option never wins because the better one has to be installed. If an install needs the owner's approval,
ask for it. The owner: "mfs recommending a less mature, unsafe, same speed alternative just because the mature, safer, maintained,
same speed needed a fucking download. And a mf put important stuff in scratchpad, and it was fucking gone. And by mfs, I mean
Claudes." (Owner, 2026-11-05)

**D71. A standard.** Cyborg has a written standard, like C's ISO one, and its behaviour is documented. Its form, and how it relates
to `DECISIONS.md`, the oracle and the compiler, are not settled here. The owner: "Cyborg would have a standard (like C's ISO), and
its behaviour would be documented." (Owner, 2026-11-05)

**D72. Decisions, then the standard, then code.** `DECISIONS.md` is the history, with why and the owner's words. The standard says what
Cyborg does now, in one place, and it's what the oracle, the compiler and check claude follow. If the two conflict, that's a bug in
the standard. design claude owns it. It has to be easy to find what is where. Claude proposed the order; the owner chose: "yea. The
standard should be easy to find what is where, and stuff." (Owner, 2026-11-05)

**D73. The standard's layout.** A `standard/` folder of numbered files, from `1-programs.md` to `12-not-decided.md`, with a `README.md`
that maps each word to its section. Sections are numbered so they can be cited. Each gives the rule, an example, what gets refused
and the decision it came from. `10-build-modes.md` holds the one table of what's UB in which mode, and anything in `12-not-decided.md`
means `NotDesignedYet`. Inside each file, a reader has to be able to find something without reading all of it. Claude proposed the
layout; the owner chose: "yea, and inside those should also be easy to find without reading the entire thing". (Owner, 2026-11-05)

**D74. `my.` for a name that clashes.** A name you define that clashes with something else is written with `my.`, as in Radium. The
owner: "if you want a thing that clashes, you write it with `my.`". Claude reads that as a clashing name without `my.` being
refused; the owner hasn't confirmed that. What counts as a clash, and whether `my.` goes on the definition, the uses or both, are not
settled here. (Owner, 2026-11-05)

**D75. `my::` for a name that clashes.** This replaces `my.` in D74. Your own names are bare, and any name is allowed, even `x` or `if`.
A name clashes when it's the same as something built into the language: a keyword, a word operator or a base type. `std::` things
don't clash, because they have a path, and variables don't, because they have `$`. The mark goes on uses, not the definition:
`fn x […]` is called with `my::x(…)`. `$a x $b` is always multiply, and a bare `x(…)` is refused with the fix shown. If Cyborg later
adds a built-in with the same name, old code gets a compile error saying to write `my::`, never a silent change of meaning. `my` is a
reserved kit prefix, and `@my` isn't reserved. From Radium's D43, with Claude proposing the rules. The owner proposed the spelling:
"should it be `my::`, and `my` is a reserved kit prefix (ironic for the thing to unreserve), `@my` isn't reserved". Claude suggested
reserving `@my` too, for looking like `my::`; the owner asked "So, what's the problem?", and Claude agreed there's no real one.
(Owner, 2026-11-05)

**D76. `precond =>` and `postcond =>`.** These replace the placeholders `requires` and `ensures` in D53, with the same order and place:

```
fn half [$n: i32] -> i32:
    precond => $n at_least 0;
    postcond => result at_least 0;
    fails [odd];
```

`=>` joins the symbols in D43, and `fails` keeps its plain `fails [odd];`. `result` is still a placeholder. The owner proposed "`precond =>`,
`postcond =>`?"; Claude suggested dropping the arrow; the owner chose: "🥹 pwease". (Owner, 2026-11-05)

**D77. Three kinds of benchmark.** Purpose: isolates one thing, and its result says what it measures, so nobody quotes it as a verdict on
Cyborg. Normal: written the way a normal person would, with no `@std::inline.asm`, shields or hand-tuning. Ceil: how fast Cyborg can
possibly go, with anything allowed. Other languages' versions follow the same rules. Purpose and normal use `FinalSafe` (D32). The owner
proposed the kinds: "those benchmarks, but strictly for their purpose, and those benchmarks normally to see how the lang does normally",
then "And another one that is the ceil"; Claude proposed the rules. The ceil's build mode is not settled here. (Owner, 2026-11-05)

**D78. Benchmarks per build mode.** Every benchmark runs in every build mode, in each of its three kinds. This replaces D32's rule that
published benchmarks use `FinalSafe`, and the build mode D77 gave purpose and normal. Any number quoted says its kind and its mode;
that part is Claude's addition. Claude asked which mode the ceil uses; the owner chose: "Actually, it's those benchmarks per build
mode." (Owner, 2026-11-05)

**D79. The repository is public.** `Soclets/Cyborg` is public, with its full history. The license and the repository's final name are still
not chosen. The owner: "Bro, pub the GH repo now". (Owner, 2026-11-05)

**D80. Imports in the file.** Cyborg imports in the file, rather than only through a manifest, so a file says what it uses. This replaces
"kits are reached by their path" with no import in D56, though a thing is still reached by its path once imported. Claude recommended
a manifest with no imports; the owner chose otherwise: "imports in file are new thing for me. So, it seems good to try out". The
spelling, what needs importing, and what a manifest still does are not settled here. (Owner, 2026-11-06)

**D81. Import plus manifest.** A file's import says what it uses. A manifest lists what's installed and which version, and versions live only
there. A program using only `std` needs no manifest, so hello world can stay a single file. Claude proposed it; the owner chose: "yea".
(Owner, 2026-11-06)

**D82. Everything is imported, and `use` exists.** `std` gets imported like everything else, and so do limbs. Builtins and `my::` need no import.
Cyborg also has something like Rust's `use`, in the owner's own style, which isn't described yet. The spelling of importing is not settled
here. Claude proposed importing `std`; the owner chose: "yea. And should it be something like `@import`? Btw, we'll also have Rust's `use`, but
in my style". (Owner, 2026-11-06)

**D83. `const`, not `:=`.** A constant is `const $b: i32 = 60 x 60;`, replacing `:=` in D58, which is gone. Its value must still be fully known at
compile time, so `const $a: i32 = read();` is refused. The owner: "Change `:=` to `const`, I shouldn't have chose that (i myself misread it)".
(Owner, 2026-11-06)

**D84. `requires` and `ensures`, again.** This replaces `precond =>` and `postcond =>` in D76, going back to D53's spelling, with no `=>`, which leaves the
symbols. `requires $n at_least 0;` and `ensures result at_least 0;` come first in the body, in that order, then `fails`. `result` is still a placeholder.
The owner: "I think we should change `precond` and `postcond` back. I have to think when I read them". Dropping the arrow with them is Claude's reading of
"back". (Owner, 2026-11-06)

**D85. C++ templates, but safer.** Cyborg has generics that make a copy per type, can take values as parameters, can be specialized for one type, and can be
extended by outsiders through bounds. They're safer because a generic says what it needs through a bound, and is checked once at its definition, so errors don't
show up deep inside someone else's code. Unchecked bodies are out, and accidental metaprogramming isn't needed, since baking would use normal code. The spelling
`<?T: ordered>` is Radium's, as a placeholder. Claude listed the features and proposed this; the owner chose: "Ah, we should have C++ Templates, but safer". Claude
reads "safer" as the bounded version. (Owner, 2026-11-06)

**D86. `fn.pure`.** A pure function always gives the same output for the same input, and changes nothing else. The compiler checks it, refusing I/O, changing anything
outside the function, `Lent.mut`, calling a function that isn't pure, shields and C calls. Contracts can only call pure functions, so turning checks off can't change
what a correct program does. Whether a pure function may allocate or fail is not settled here. Claude proposed it; the owner chose: "yes, I want a pure fn". (Owner,
2026-11-06)

**D87. A type declares its bounds.** A type fits a bound only if it says so, even if it can already do what the bound needs. So a type can't fit by accident. The spelling
`reading is ordered` is a placeholder. How a bound is defined, and whether you can declare one for a type from someone else's kit, are not settled here. Claude suggested
automatic; the owner chose: "declared". (Owner, 2026-11-06)

**D88. A separate language, written inline.** Cyborg's equivalent of C++ templates is a separate compile-time language written inline in Cyborg code, more powerful than templates and safe.
This goes further than D85, which stands for what generics must do. Claude suggested staying with bounded generics and baking, adding features when needed; the owner chose: "C++ Templates
equivalent, a separate language written inline, but more powerful and safe". Its syntax, what it can do, what "safe" requires, and how it relates to baking are not settled here. (Owner,
2026-11-06)

**D89. What "safe" means for the inline language.** If compile-time code passes the compiler, every program it can generate is valid Cyborg, with all of Cyborg's rules. So it's checked at the
definition, generated code follows ownership, `%`, shields and everything else, it always finishes, such as through a step limit that gives `refused:`, and errors point at real code, with
`because:` leading back to the compile-time code. Claude proposed this strong version and a weaker one that checks at the use; the owner chose: "yea". Claude reads that as the strong version.
(Owner, 2026-11-06)

**D90. What the inline language can do.** It commits to picking a type by a condition, any number of type parameters, and looping over a struct's fields, with each field's type fitting a bound.
Generating a struct's fields and generating functions are goals, if the checking in D89 works out. Generating arbitrary code from text is out, because D89 can't hold for it. Claude proposed it; the
owner chose: "yea". (Owner, 2026-11-06)

**D91. The inline language does all compile-time work.** Types, structure and values. A `const`'s value is inline-language code, and `bake` is the shortest piece of it, as in `let $day: i32 = bake
seconds_in(1);`. It can call normal functions marked `fn.pure.baked`, and only those, so nothing gets written twice, and those functions also run normally. Breaking one gives an error on that function.
All compile-time code gets D89's guarantees, including the step limit. The owner asked "Why not let the inline lang do it?" about Claude's suggestion of two separate tools; Claude agreed and proposed this;
the owner chose: "yea". (Owner, 2026-11-06)

**D92. `import std;`.** Importing is a plain keyword, with `import @csup;` for a limb. A thing is still reached by its path, as in `std::println`. It goes at the top of the file, so a file's dependencies can be read
without running anything. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-06)

**D93. `use … as …;`.** `use` gives a path a shorter name for that file, and `as` is always written:

```
use std::print.useful as printu;
use @csup::extern.c as ext;
```

The narrowing `std::print.useful` and `extern.c` are the owner's examples. Whether a limb's short name keeps its `@` is not settled here. The owner: "Like that". (Owner, 2026-11-06)

**D94. You can't `use` a limb.** A limb is always written with its full mark and path, so D22's promise holds: a search for `@` finds every use of a compiler extension. This replaces the second example in D93, so
`use @csup::extern.c as ext;` is refused. Claude suggested short names that keep their `@`; the owner chose: "Or just make it so that, you can't `use` a limb". (Owner, 2026-11-06)

**D95. Printing can fail, and failures can't be ignored silently.** `std::println` is a normal function that can fail. A call to anything that can fail must be handled: `?` hands it to the caller, `?? void` ignores it on
purpose, and `otherwise:` handles it. So `std::println("hi");` is refused, and hello world says `std::println("Hello, world!") ?? void;`. The failure's name `print_failed` is a placeholder. The owner asked "If print can fail,
why not `?` at the end?"; Claude suggested a bare `println` that ignores failure, with a `.checked` narrowing; the owner asked "don't we usually like marks?"; Claude proposed `?? void`; the owner chose: "yea". (Owner, 2026-11-06)

**D96. A failure crashes by default, and handling is opt-in.** A call that fails with nothing handling it stops the program loudly. You opt in with `?`, `??` or `otherwise:`. This replaces the part of D95 that refused an unhandled call, so `std::println("hi");`
is allowed, and crashes if printing fails. `?? void` still ignores a failure on purpose. A full list and running out of memory are failures like any other, and so is a failed `as`. The crash stays in every mode, since it's a real runtime event, as Claude
proposed without objection; `FinalShit` isn't settled. The linter limb can be stricter, such as requiring handling. The owner: "in my other serious langs, we default to crash, and then opt-in handle", then "every failure, then the linter limb could make it
more strict". (Owner, 2026-11-06)

**D97. No bare `mod`.** Like `div` (D48), a remainder has no single right answer for negative numbers, so you pick: `-7 mod.trunc 2` gives -1, and `-7 mod.floor 2` gives 1. This replaces the bare `mod` in D49. Claude proposed it; the owner
chose: "yea". (Owner, 2026-11-06)

**D98. A full print, like Zig's.** Besides the short `std::println`, `std` has a way to do the whole thing by hand, as Zig's long stdout version does: choosing the stream, the buffer and when to flush, with failures handled. Its spelling is not settled
here. The owner: "We should have a way to do Zig's full print". (Owner, 2026-11-06)

## Open

- **The test in D9 on a type that's already known.** `if $n: i32[min: 0, max: 100]:` works on a plain `i32` (D20). Claude
  suggested refusing `if $n: bool:` on a value already known to be one, since it's always true or always false. Not
  chosen yet.
- **Function types.** Since `-> i32` no longer shows failures (D52), a function's type needs some way to include them.
- **Radium's open ownership questions, inherited with D2.** Lending through a stored `Lend.mut`; how a `must_Give` duty
  ends in the function that receives it; a `must_Give` value in a field or returned; a parameter whose type is a stored
  borrow; a `union.tagged` case holding a borrow; going through items that are structs, unions or arrays.
- **Errors, left open by D16.** Colour, how many errors per run and their order, and any machine-readable form.
- **More on `@`.** Whether the mark goes only at the path's start; whether other limbs have a kit part like `std`; and how a
  third-party kit or limb is spelled.
- **The keyword `shield`.** Radium's working word, not settled there either.
- **`d32`'s limb.** D55 left it open. The owner: "Could just be a money limb, we don't need to design now". Deferred.
- **Everything in `standard/12-not-decided.md`.** Written with the standard, so that's the full list, including the problems found
  writing it. This section may be behind it.
- **Placeholders.** The repository's name, the license and the file extension are not chosen.
