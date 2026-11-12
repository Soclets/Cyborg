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

**D99. A number says its unit, behind it where possible.** `3600 seconds`. Where the unit can't go behind, it goes in front. Claude proposed units inside names, like `time_in_seconds`; the owner chose: "I meant unit behind where possible, if not just do unit in front.
`3600 seconds`". Whether units are checked by the compiler, which numbers need one, and how the examples in the standard change are not settled here. (Owner, 2026-11-06)

**D100. The compiler checks units.** A unit is part of the type, carried by a narrowing, as in `$a: i32.seconds`. Starting small: the same units can add and compare, mixing them is refused, and anything fancier, such as `seconds x seconds`, waits until a real program
needs it. Claude proposed it; the owner chose: "Oh, that is a good fucking idea". (Owner, 2026-11-06)

**D101. Which numbers need a unit.** Only where the type asks for one. A plain `i32` means just a number, and needs none. The language and `std` ask for units on things that have them, like lengths, sizes and durations, and your own code opts in with a type like
`i32.seconds`, so `let $b: i32.seconds = 3600;` is refused. So `std::array<u8>[len: 4096 items]` and `len.max: inf items`. The linter limb can be stricter. Claude proposed it; the owner chose: "yea". Renaming `@size_of` to `@bytes_of`, with the unit in front, is Claude's
reading of D99. (Owner, 2026-11-06)

**D102. A collection's first value.** `[1, 2, 3]` gives a list or array its items, with the count checked against the type. `= all 0` fills an array with one value, and works with other values too, as in `= all 255`. `= empty` stays for an empty list, and a list with `len.min`
above 0 can't use it. Claude proposed `[1, 2, 3]` and `= zeroed`; the owner asked "What about `all 0`? That seems more explanatory", and raised nothing against `[1, 2, 3]`. An uninitialized array, like Zig's `= undefined`, is not settled here. (Owner, 2026-11-06)

**D103. `empty` means nothing in here, for strings too.** `let $s: str = empty;`. `""` and `[]` are refused, with `fix: empty`, because `""` could look like unfinished work. `empty` is refused where it can't be true. The owner: "we literally have to say `return void`, and `""` could
very much look like unfinished work"; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-06)

**D104. A test's trust ends at a `change`.** Inside `if $s: status.failed:`, changing `$s` is allowed, and the compiler stops trusting that `$s` is that kind from there on. Anything after it that needs that trust is refused, with `because:` pointing at the `change`. The same goes for
lending it with `Lend.mut`, which is Claude's reading. Claude proposed it over refusing the `change`; the owner chose: "yea". (Owner, 2026-11-06)

**D105. `FinalSafe` keeps every contract, unless you mark it.** A narrowing on `requires` or `ensures` says a contract is checked in `Working` only. A shield's exit stays checked in `FinalSafe` anyway (D37), and Precog can still drop what it proves. The word isn't chosen; Claude's `.expensive` is a
placeholder. Claude proposed it; the owner chose: "yes. But I think it should be another word". (Owner, 2026-11-06)

**D106. `.only_in_working`.** `requires.only_in_working is_sorted(Lend $xs);`, and `ensures.only_in_working` likewise. This settles the word D105 left open. Claude suggested `.slow`; the owner proposed: "`requires.only_in_working`?", and Claude agreed, since it says exactly what it does. (Owner, 2026-11-06)

**D107. `FinalShit` keeps only what it needs to generate code.** Syntax, known names and types that fit. Everything else that exists to stop you doing something dumb is gone: ownership words, `%`, `~`, `*`, units, and covering every case in a `match`, where a missed case is UB. An unhandled failure is UB,
not a crash, since it assumes nothing ever fails, and `?` and `otherwise:` may get deleted too. This settles what D40's "such as" and D96 left open. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-06)

**D108. You can define units, and they do more.** You can make your own units, say which ones go together, define conversions, and give them narrowings. A unit from a kit is written with its path, `name::unit`. This goes past D100's "starting small". Their spelling is not settled here. Claude proposed a one-line `unit pixels;`
with no conversions; the owner chose: "what about more? Like saying which could go together, conversions, and narrowings for them too. And of course, a unit from a kit uses `name::unit`". (Owner, 2026-11-06)

**D109. How a unit is defined.** Units live in a collection, `narrowings.unit`. Every unit says what it is, either `: base` or `=` with a conversion, and a bare line is refused. `=` can point at a unit in any family, including one from a kit. Two units convert if they trace back to the same base, and otherwise they don't,
so incompatibility is written down rather than left out. A family is only a group of related names that a generic can ask for. Loops and units with no base are refused. The owner: "I mean, we do say `return void`, etc", then "What if you want incompat but not base, and compat with another collection?"; Claude proposed this;
the owner chose: "yea". (Owner, 2026-11-06)

**D110. Unit maths, offsets, and points.** Cyborg goes further with units: the compiler works out new units from maths, like `meters` over `seconds`; conversions can be offsets, not only multiplying, as with Celsius and Kelvin; and a moment is kept apart from a length of time, so `3 PM + 2 hours` makes sense and `3 PM + 3 PM` doesn't. These
are levels 3 and 4 from a list the owner's friend sent. Their spelling, and how they fit with D108's "which could go together", are not settled here. The owner: "The ones we don't have yet, I think 3 & 4 are pretty useful". (Owner, 2026-11-06)

**D111. Unit maths works where you say.** Multiplying or dividing two units only works where a `together` line declares it, and the line names the result: `together meters / seconds = meters_per_second;`. The compiler then works out the unit in the maths, as D110 wanted. `meters x dollars` is refused unless declared. Claude
suggested letting all unit maths compile, with `together` only naming results; the owner chose the other way: "The `return void` vibe". The line's spelling is a placeholder, and naming the result is Claude's reading of that vibe. (Owner, 2026-11-06)

**D112. A unit always says its family.** `let $a: i32.time.seconds = 3600 time.seconds;`, the same way a handler is `word_failed.use_placeholder` and never bare. From a kit, it's `3600 std::time.seconds`, and `use` can shorten that. This replaces the short `3600 seconds` in D99's example, though the unit still goes behind the number.
`items`, as in `len: 4096 items`, is base language and stays bare, which is Claude's reading of "the base is bare". Claude suggested the short name; the owner chose otherwise: "look at handlers, you don't say `flow_back` bare, you say `cum_failed.flow_back`. And yes, that is an official example in CyborgPL". (Owner, 2026-11-06)

**D113. A declared label must be written.** A function's parameter can have a label, and then every call writes it. A label is separate from the parameter's own name, so `fn append [val $item: i32]` is called `append(val: 4)`, and a parameter with no label is called without one, as in `total(Lend $xs)`. So each function has one way to call it. An
ownership word goes after the label: `append(val: Give $r)`. The declaration's spelling is a placeholder. The owner: "You don't have to name the args, you have to name the label". Claude reads that as the function's writer deciding which parameters have labels. (Owner, 2026-11-06)

**D114. The `labels` line, from Radium.** A label is declared at the top of the body: `labels [$item as val];`, called as `append(val: 4)`. This replaces the spelling in D113. The top of a body is now `labels`, `defaults`, `requires`, `ensures`, `fails`, then the code, extending D53. `defaults [$limit = 40];` makes a labelled parameter optional, and
only labelled parameters can have defaults. Labels keep their declared order, and `labels.any_order` lets a caller reorder them. In `std`, the first argument, the thing being worked on, stays bare, and every other argument gets a label, except lists where every item has the same role, like `print`'s pieces. Your own functions label as you like. Claude
proposed taking it whole; the owner chose: "yea". (Owner, 2026-11-06)

**D115. A method from a kit says its path.** `$xs.std::%append(val: 4);`. Bare means base, or your own, so `$xs.%append` would wrongly look like one of those. This applies the rule that everything else follows: base and your own are bare, a kit has its path, and a limb has `@`. Your own method stays `$p.%area()`, and a limb's is `$x.@csup::%name()`. The `%` still sits on
the method's name. Claude suggested a bare method for a type's own kit, because the type already says where it comes from; the owner asked "Wouldn't it be `.std::%append`?", then "doesn't bare mean the base's?", and Claude agreed. (Owner, 2026-11-06)

**D116. Precedence follows maths, and `( )` groups.** Tightest first: `.`, `.%`, `::`, calls and `?`; then `not` and unary `-`; then `x`, `div…` and `mod…`, left to right; then `+` and `-`; then comparisons; then `and` and `or`; then `??`. Where maths gives no order, you group with `( )`, D23's one exception: bit operators and shifts mixed with anything, and `as`
on anything but a single value. Claude proposed the table and refusals; the owner chose: "Follow maths, group with `()` (exception of bracket rule)". Whether `and` binds tighter than `or`, and whether `$a < $b < $c` is allowed, are not settled here. (Owner, 2026-11-06)

**D117. `and` over `or`, and comparison chains.** Following maths, `and` binds tighter than `or`, so `$a and $b or $c` means `($a and $b) or $c`. `1 < $x < 5` means `1 < $x and $x < 5`, with `$x` evaluated once. The linter limb can ask for parentheses. Claude proposed it; the owner chose: "yea, and the linter would be real". (Owner, 2026-11-06)

**D118. `//` and `///`.** `//` is a line comment, a note for whoever reads the code. `///` is a doc comment, attached to what's below it, which tools can show and build documentation from. Block comments are not settled here. Claude proposed both; the owner chose: "Both". (Owner, 2026-11-06)

**D119. Named comment blocks.** A comment spanning lines is a block with a required name: `comment why_sorted:` to `end comment why_sorted`. Blocks nest, and a closer must name the block it closes. Both lines count only when alone on a line. `//` needs a space after it, and `///` is its own form. From Radium's D86 and D87, where `comment:` was the
owner's design; nesting and names are new. The owner asked "Name the comment block?"; Claude proposed requiring it; the owner chose: "yea". (Owner, 2026-11-06)

**D120. Declarations work in any order.** A function can call one declared later in the file, with no forward declarations. The linter limb can enforce an order for anyone who wants one. Oracle claude asked; Claude proposed it; the owner chose: "yea, and I mean, the linter could choose anyways". (Owner, 2026-11-06)

**D121. A closer repeats all of it.** `fn.pure.baked seconds_in` closes with `end fn.pure.baked seconds_in`, like `end fn.main` and `end union.tagged status`. This corrects the examples in D86 and D91, which Claude copied from Radium. Oracle claude found the conflict; Claude proposed following the rule; the owner chose: "yea". (Owner, 2026-11-06)

**D122. A number prints bare.** `$xs.len as str` gives `3`, with no unit. A narrowing asks for the unit, spelled `as str.with_unit` as a placeholder. How the unit then prints, such as `3600 time.seconds`, is not settled here. Oracle claude asked; Claude proposed it; the owner chose: "Print bare, and a narrowing?". (Owner, 2026-11-06)

**D123. Every plain number takes no ownership word.** Plain numbers and `bool` are always copied, covering `i32`, `f64`, bounded integers, numbers with units and `d32`. This widens D2, which came from Radium before it had floats and only named integers and `bool`. On a number, a word can't tell you anything you could be wrong about. Oracle claude asked;
the owner asked "Does saying the word really add any safety?"; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-06)

**D124. `and` and `or` stop early.** The right side only runs if it's needed, so `$n != 0 and 10 div.trunc $n > 1` is safe. A comparison chain stops at its first false too. A narrowing runs both sides, spelled `and.both` and `or.both` as placeholders. Oracle claude asked; Claude proposed stopping early; the owner chose: "Stop early, and a narrowing to
run both". (Owner, 2026-11-06)

**D125. `change` works on a field.** `change $r.celsius = 22;`. The whole variable has to be `let mut`, or the parameter `Lent.mut`, and single fields aren't marked as changeable. Oracle claude asked; Claude proposed it; the owner chose: "yea". (Owner, 2026-11-06)

**D126. `else` and `else if`.** `else if $n > 0:` and `else:` continue an `if`, with one `end if` closing the whole chain. The type test works in `else if` too. Oracle claude said it was missed most; the owner asked "Where's else if?"; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-06)

**D127. `true` and `false`.** A `bool` is `true` or `false`. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-06)

**D128. Radium's loops.** Each loop says how it ends: `for` goes through something, `while` runs while a `bool` holds, and `forever` has no end unless you `break`, so one word finds every endless loop. A range is `range[from: 1, to: 5]`, where `to:` includes the end and `before:` stops before it, with `step_by:` after them, and a negative step
counts down; `from:` is required, the order is fixed, and a range that can never reach its end is refused. `for`'s own value can't change inside. `break;` stops the innermost loop, and a loop can be named, as in `for rows $row in $grid:` closed with `end for rows`, so `break rows;` stops it and everything inside it. There is no `continue`, which
removes it from D46's list. `decreases` belongs to the Precog limb. The owner asked "what's the difference?", then to ask Radium and TAKL's sessions; Claude proposed taking Radium's; the owner chose: "yea". (Owner, 2026-11-06)

**D129. `finish` ends a `must_Give` duty.** `finish $f;` says the duty stops here. Only the kit or file that declared the type can write it, or anyone could end the duty without doing the work, and it's refused until any `must_Give` fields inside have been given away. This settles what Radium left open in D2. Whether a C function's `Given` parameter ends
the duty too is not settled here. Oracle claude said it blocked the most; Radium's session had no ruling, and leaned toward taking the struct apart; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-06)

**D130. Declaring a method.** A method is a normal function with its type in the name: `fn reading.%warm_up [$r: Lent.mut reading, $by: i32] -> void:`, closed with `end fn reading.%warm_up`, and called `$r.%warm_up(by: 2);`. The first parameter is the receiver, and says how it's held, which is where `%` looks up what's unsaid. A bare one is read-only, and
`Given` is refused there, since methods only borrow (D8). Claude proposed it; the owner chose: "yea". (Owner, 2026-11-06)

**D131. `not` sits below comparisons.** `not $a < 3` means `not ($a < 3)`, which corrects D116. `not` now sits just above `and`, and unary `-` stays where it was. Oracle claude found it; Claude proposed the fix; the owner chose: "yea". (Owner, 2026-11-06)

**D132. `??` sticks to its call.** `??` sits level with `?`, so `$a + read() ?? 0` means `$a + (read() ?? 0)`, correcting D116. Its right side is the next single value, so `read() ?? 1 + 2` means `(read() ?? 1) + 2`, which is Claude's reading. Oracle claude found it; Claude proposed the fix; the owner chose: "yea". (Owner, 2026-11-06)

**D133. `pos:` and `offset:`.** `$xs[pos: 1]` is the first item, counting from 1, and `$xs[offset: 0]` is also the first item, counting from 0, so the word says which you mean. `change $xs[pos: 3] = 10;` changes one, and needs what D125 needs. A bad position is a bounds check, as in D26: a trap in `Working` and `FinalSafe`, and gone in `FinalPerf`, which is Claude's
reading. Claude proposed both, from TAKL; the owner chose: "yea, Ra has `offset` too". (Owner, 2026-11-06)

**D134. Naming a type.** `type.alias score = i32;` is just another name, fully interchangeable. `type.new user_id = i64;` is the same data but a different type, so it doesn't mix with another `type.new`, or with `i64`. There's no bare `type`, since no default is obvious. A family of them is `narrowings.type.alias sizes:` or `narrowings.type.new ids:`, with lines like
`ids.user = i64;`, always used with the family, and a family doesn't mix the two kinds. D10 still holds for the language itself, so `int.signed.32_bit` doesn't exist unless you write it. The owner: "We should have a `type i64 = int64` style narrowings", then "Yes, but I also mean collections too"; Claude proposed the spelling; the owner chose: "yes". (Owner, 2026-11-06)

**D135. `fam.` for a family.** A declared group of related names is a family, written `fam.unit`, `fam.failure`, `fam.handler`, `fam.type.alias` and `fam.type.new`, replacing `narrowings.…` everywhere, including `fn` collections. "Narrowing" now only means the `.`. The owner: "should we just call them families, and syntax is `fam.`? Because like, narrowings could
mean a lot, even in our context only", then "yea". (Owner, 2026-11-06)

**D136. The pitch.** "Immature, uncertified, clear and C-family flavoured SPARK". Claude suggested "Nothing goes unsaid.", then questioned "C-family", since Cyborg has `end if` and no braces; the owner answered: "I mean, most people consider a lot of symbols and brackets as C family". Cyborg isn't proven by default the way SPARK is, and "uncertified" covers that. Keeping the owner's wording as
it stands is Claude's reading of that answer. (Owner, 2026-11-06)

**D137. The extension is `.cy`.** This settles D16's placeholder. GitHub's Linguist files `.cy` under Cycript, and Cyborg has no Linguist entry, so `.gitattributes` says `*.cy linguist-detectable=false`, as CyborgPL's did. Claude read a joke about Cycript as wanting a new extension; the owner: "Bro, `.cy` is it. What I meant is look at linguist". (Owner, 2026-11-06)

**D138. `map` has narrowings.** `std::map.hash<key: str, val: i32>` and `std::map.ordered<key: str, val: i32>`, so you pick. There's no bare `std::map`, since no default is obvious, which is Claude's reading of D48's rule. The names are placeholders. This changes the examples in D24 and D25, though their labels stand. Claude listed `map` as a word with unstable meaning; the owner: "Sherlock, we're gonna fucking make a narrowing of `map`". (Owner, 2026-11-07)

**D139. Reading a failure, and `instead`.** `case negative $f:` names the failure inside its arm, so `$f.val` reads its data, and `case status.failed $f:` does the same in a `match`. `instead 0;` is the value used in place of a failed call, replacing Radium's bare last line. An arm can also `return`, `break` or `fail`. The owner asked "What does `0;` mean?"; Claude agreed a bare value reads like a leftover, and proposed this; the owner chose: "`instead 0;`". (Owner, 2026-11-07)

**D140. A list decides its length's type.** `$xs.len` is `u64.items` by default. A list can choose another with `len.type`, as in `std::list<i32>[len.max: inf items, len.type: u32]`, which also changes how the list stores its length, and then `$xs.len` is `u32.items`. A `len.max` that doesn't fit is refused. An array's `$a.len` follows the same rule, which is Claude's reading. Claude
suggested `u64.items` plus narrowings on each read; the owner asked "Why not unsigned?", "Why not let them choose?", then "Ah, so decide on the `list`?", which Claude reads as this. (Owner, 2026-11-07)

**D141. `as` crosses, and literals don't need it.** A literal takes its type from where it lands: `let $u: user_id = 5;`, and `let $t: u64.items = 5 items;`. A value that already has a type crosses with `as`: `$n as user_id` and `$u as i64` for a `type.new`, and `$xs.len as u64` to take a unit off. This can't fail, as it's the same data. Oracle claude said `type.new` couldn't produce a value, and a length couldn't become
a plain number; Claude proposed `as`; the owner asked "Why does `5` need to say `as`?", and Claude agreed; the owner chose: "yea". (Owner, 2026-11-07)

**D142. Positions take a length.** `pos:` and `offset:` accept a number in `items` or a plain one, so `$xs[pos: $xs.len]` gets the last item, and `$xs[pos: 3]` still works. Ranges do the same: `range[from: 1, to: $xs.len]`. Claude proposed it; the owner answered: "Why not?", which Claude reads as a yes. (Owner, 2026-11-07)

**D143. `offset:` says `items`, and `pos:` doesn't.** `$xs[offset: 8 items]` says its unit, because an offset could read as bytes, as with `len: 4096 items`. `$xs[pos: 3]` doesn't, and `$xs[pos: $xs.len]` still works, since nobody reads position 3 as a byte. This narrows D142 for `offset:`. The owner: "`len: 4096` kinda look like bytes to me", and "`pos: 3 items` reads weird"; Claude proposed the split. (Owner, 2026-11-07)

**D144. A label can carry a unit.** A label's narrowing can name a unit, so `$xs[offset.items: 8]` replaces D143's `offset: 8 items`, with no bare `offset:`. A plain number takes the label's unit, a value with the right unit is fine, and a value with another unit is refused, needing `as` if they convert. It works with your own units too: `labels [$x as at.screen.pixels];` is called `draw(at.screen.pixels: 5);`. The writer picks this or
a unit behind the number when declaring a function, so each function has one way to call it. Claude proposed it, since `offset: 8 items` "reads weird"; the owner chose: "yea". (Owner, 2026-11-07)

**D145. A range that can't reach its end runs zero rounds.** When its numbers are only known while running, as with `range[from: 1, to: $xs.len]` on an empty list, the body doesn't run and there's no error. A step of 0 is a failure, since it never ends. Literals that can't reach their end are still refused, as in D128. The cost is that a range written wrong by accident does nothing silently, and the linter limb can flag that. Oracle claude asked; Claude proposed it; the owner
chose: "Yea. I mean, the linter limb could help". (Owner, 2026-11-07)

**D146. `step_by:` takes any integer.** A literal, or any signed or unsigned integer, and it doesn't have to match the range's type, so `range[from: $xs.len, to: 1, step_by: -1]` works, and `$i` keeps the range's type. Oracle claude asked; Claude proposed it; the owner asked "Should they choose?", and Claude said there's nothing to choose, since a step isn't stored; the owner chose: "yea". (Owner, 2026-11-07)

**D147. A loop knows its range.** Inside a `for` over a range, the compiler knows the value is within that range, and skips any check it can already see will pass, as with `$xs[pos: $i]` in `range[from: 1, to: $xs.len]`. It needs no new syntax. If the loop could make that untrue, such as by removing an item, the check stays. The owner asked "What about bounds on loop ranges?"; Claude proposed this; the owner chose: "Ah, yeah". The owner's own idea of putting
bounds on the ranges is not settled here. (Owner, 2026-11-07)

**D148. No bounds inside a range.** `if $start: i32[min: 100, max: 200]:` or a bounded `let` already do it, and D147 then skips the same checks. `$xs.len[min: 100]` would also look like getting an item (D133). The owner proposed bounds inside a range, then asked "Does `if` already work for that, though?"; Claude said it does; the owner chose: "yea". (Owner, 2026-11-07)

**D149. `as` on a whole value says its ownership.** `Give $r as wrapped` leaves `$r` dead, and `Copy $r as wrapped` leaves it alive and pays for a copy, following D3. A plain number needs no word (D123), and neither does a fresh value. This covers structs, lists and strings in a variable. Oracle claude asked; Claude proposed it; the owner chose: "yea". (Owner, 2026-11-07)

**D150. Checked `use`.** An unquoted name has to match the last word of its path, so a short name can't lie: `use std::time.seconds as seconds;` is fine, and `use std::time.seconds as minutes;` is refused. A quoted name is your own choice, and isn't checked: `use std::time.seconds as "secs";`, used afterwards as `3600 secs`. This covers every `use`, so D93's `use std::print.useful as printu;` becomes `as "printu"`. The quotes only appear on the `use` line, which is Claude's reading. The owner: "`seconds` without `""` matches the actual unit there"; Claude proposed extending it to every `use`; the owner chose: "yes". (Owner, 2026-11-07)

**D151. A quoted name still has to be usable.** The quotes only skip the check against the path. `use std::println as "my say";` and `use std::println as "";` are refused, since nothing could call them, and so is a keyword like `as "fn"`, which is Claude's reading. Oracle claude asked; Claude proposed refusing them; the owner chose: "yea". (Owner, 2026-11-07)

**D152. What Cyborg is for.** A general systems language with a very safe syntax, on the safer side, at around top performance, where safety wins if performance drops a little, and with good C++ support. It won't seek a real certification. This sharpens D27. The owner: "what I wanted was just a hella safe syntax, a general sys lang that's on the safer side, on around the top perf (safety wins if perf drops a little, even though I know some AIs won't note the important nuance), and good C++ support (for obvious reasons), and I wouldn't go for a real cert". Whether D31's "no deep C++" still holds, and whether the pitch in D136 still fits, are not settled here. (Owner, 2026-11-07)

**D153. You can `use` a limb, keeping its `@`.** `use @csup::extern.c as @c;` then `@c …`. This replaces D94. The short name has to keep the `@`, so a search for `@` still finds every use of a compiler extension, and D150's check applies, so your own name is quoted, as in `as "@ext"`. The path is one look away at the file's `use` lines. Claude proposed it; the owner answered: "They could just look at the `use`s, right?", which Claude reads as a yes. (Owner, 2026-11-07)

**D154. You can't `use` a limb, again.** This replaces D153 and restores D94: `use @csup::extern.c as @c;` is refused. `@c` looks like a builtin (D56), or like a limb called `c`, so the short name misleads. Claude had read "They could just look at the `use`s, right?" as a yes when it wasn't one. The owner: "I mean, `@c` looks like a lie". (Owner, 2026-11-07)

**D155. `use` on a limb works like a kit.** `use @csup::extern.c as c;`, or `as "ext";` for your own name, under D150's rules, with no `@` on the short name. This replaces D154 and D94. It changes D22's promise: a search for `@` now finds every file using a limb, through its `use` line, and not every line. Anything dangerous is still inside a shield (D38), and the linter limb can require the full path. The owner: "Or we could just allow it without the marks, like what we already allow for kits", and "What about the `""` too?"; Claude proposed one rule for every `use`; the owner chose: "yea". (Owner, 2026-11-07)

**D156. C only, for now.** `@csup` holds C: headers read directly, C's layouts, calls through C's conventions, and generated `.h` files so C and C++ can call into Cyborg by name. There's no C++ support, and a `@cppsup` limb could be added later without changing the language. This corrects D31, which came from a wrong summary of CyborgPL, and "good C++ support" in D152. CyborgPL's design session confirmed it had only C. Claude proposed this; the owner asked "Or do we fuck C++?", and chose: "yea". (Owner, 2026-11-07)

**D157. The pitch, again.** "General sys lang that loses a little perf for safety." This replaces D136, since Cyborg isn't about proving things the way SPARK is. The "a" matters: it admits a cost, and nothing's been measured yet. The owner proposed it without the "a", calling it a "GAIM trap", and Claude pointed out the difference; the owner chose: "with "a"". (Owner, 2026-11-07)

**D158. Radium-style floats.** `f64[min: -40.0, max: 125.0]` has bounds, like integers. A float can't be NaN or infinity: making one is a failure, a runtime check in `FinalSafe`, dropped in `FinalPerf`, and C's floats are checked as they come in through a shield. A float gives the same bits on every machine by default. Narrowings opt out: `f64.ieee` allows NaN and infinity, and `f64.fast` allows merged and reordered maths, both placeholder names. Comes from Radium's
D24 and D153, which were never built there. Claude proposed it; the owner chose: "you know I'm accepting all of that". Whether `f64.ieee` needs a shield, and whether floats get a bare `div`, are not settled here. (Owner, 2026-11-07)

**D159. `f64.ieee` needs no shield.** A NaN is a wrong number, not something that can corrupt memory, and the compiler knows how `f64.ieee` behaves. It's a different type, so it can't mix with `f64`, and `$reading as f64` is a real conversion that fails on NaN or infinity. The owner asked "Should `f64.ieee` be in a shield?"; Claude proposed not; the owner chose: "yea". (Owner, 2026-11-07)

**D160. Comparing floats.** `-0.0 == 0.0` is true, and every other comparison follows: `-0.0 < 0.0` is false, and a bound stopping at `0.0` lets `-0.0` through. Comparing with `==` works on floats. Rare code that cares about the sign can check it directly. The linter limb can flag these cases, like comparing floats with `==`, or a range that runs zero rounds. How NaN compares on `f64.ieee` is not settled here. Oracle claude asked; Claude proposed it; the owner chose: "yea, and the linter would have a say on that too for both, and more if needed". (Owner, 2026-11-07)

**D161. `std` is flat, floats print shortest, and families are one level.** `std::sin($x)`, not `std::math.sin($x)`, the same as `std::println`. If `std` ever groups things, that's `::`, as in `std::math::sin`, and not `.`. A family is one level deep, so a family can't hold a family; `std::time.seconds` is fine, since `std` is a kit. `std::sin` is backed by CORE-MATH inside a shield in `std`, which needs a narrow exception to rule 1: third-party C that `std` needs, in its own folder, with its original license, and no C of your own. Its license and code haven't been checked yet. `std::sin.fast` is
a placeholder for a version that isn't correctly rounded. A float prints as the shortest text that means its value, so `~0.1` prints `0.1`, and `std::println.stored($x)` prints what's actually in memory. The owner proposed `println.stored`, and said of `math`: "which isn't my taste, I'm not fucking Zig... it might wanna be with `::`"; Claude proposed the rest; the owner chose: "yea". (Owner, 2026-11-07)

**D162. `std::println.stored` prints every digit.** There's no limit, so `~0.1` as an `f64` prints all 55 digits: `0.1000000000000000055511151231257827021181583404541015625`. A limit would stop showing what's stored. This corrects the example in the standard, which stopped at 34 digits by Claude's mistake. Oracle claude asked; the owner said "`std::println.stored` is honest", Claude asked whether that meant every digit, and the owner chose: "yea, no shit". (Owner, 2026-11-07)

**D163. `f64.fast` is exempt from rule 5.** The oracle runs `f64.fast` plainly, in the order written. Its tests only check that the answer is close, not identical, and a crash caused only by merged or reordered maths isn't a bug: that's what writing `.fast` allows. Every other type still matches byte for byte. How close counts as close is not settled here. Oracle claude found the conflict; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-07)

**D164. Floats get a bare `div`.** `7.0 div 2.0` is `3.5`, since a float has no remainder to throw away, and it rounds to the nearest float like `+`, `-` and `x` do. Integers keep no bare `div` (D48), so `7 div 2` is still refused, with a fix naming `div.trunc` and `div.floor`. Dividing by `0.0` makes infinity, which is a failure on `f64` (D158). Whether integer narrowings like `div.trunc` work on floats, and what `mod` does on them, are not settled here. Oracle claude asked; Claude proposed it; the owner asked "why?" and then chose: "yea". (Owner, 2026-11-07)

**D165. Exponent literals say so.** A float literal can have an exponent, and the word goes in front of the value: `let $avogadro: f64 = exponent ~6.02214076e23;`. `exponent` is a placeholder name, and an `e` without it is refused, so a stray `e` can't quietly change a number. `~` still follows D59. It's still a plain `f64`, and it doesn't remember how it was written, so `$avogadro as str` prints plain digits however long, with no switch at any length. Printing with an exponent is left to format print, which the owner expects to exist and which isn't designed yet. Negative exponents and exponents on integers are not settled here. Claude proposed exponents and a 21-digit switch for printing, then dropping the word; the owner said "it isn't really an "exponent float", maybe it should be infront of the val?", then "the word does add `use`'s `""` level safety", and chose: "yea". (Owner, 2026-11-07)

**D166. How NaN and infinity print.** On `f64.ieee`, they print as the words `NaN`, `inf` and `-inf`, since they have no digits to show. `std::println.stored` prints the same words. Oracle claude found the gap; Claude proposed it; the owner chose: "yea". (Owner, 2026-11-07)

**D167. IEEE 754 is the authority on `f64.ieee`.** The name is the promise, so IEEE 754 decides how it behaves: comparing, arithmetic, and NaN and infinity moving through the maths. So any comparison with NaN is false except `!=`, and `NaN == NaN` is false. Cyborg still decides what IEEE doesn't cover, such as syntax, printing (D166), `as` crossings (D159) and bounds. IEEE doesn't fix every bit of a NaN, so two machines can differ in bits only a raw look at the bytes would show. Claude proposed keeping IEEE's comparisons; the owner asked "It says IEEE, so IEEE here is the authority?", Claude proposed this rule, and the owner chose: "yes". (Owner, 2026-11-07)

**D168. What the first compiler comes with.** The first compiler has LLVM's standard optimizations through Lyftar, such as inlining, constant folding, dead code removal, loop unrolling and vectorizing, and Cyborg's own, where Precog drops checks it proves. It also compiles fast from the start: incremental, so only what changed is rebuilt, parallel across every core, cached between builds, and with a lighter `Working` build that runs fewer passes. This extends rule 10. The owner said "Things that should come in the first compiler: standard general optimizations, and fast compilation tricks"; Claude proposed the list; the owner chose: "Yeah". (Owner, 2026-11-08)

**D169. A stored borrow ends at its last use, or says `.held`.** `let $view: Lend from $r reading = Lend $r;` ends at the last use of `$view`, so `$r` can be changed after it, like Rust since 2018. `Lend.held from $r reading` lasts until the end of its block even if it's never used again, so the code says nobody touches `$r` for the whole block. `Lend.mut.held` works the same way. `held` is a placeholder name. Claude proposed last use; the owner asked "How about let them choose?", Claude proposed the narrowing, and the owner chose: "yea". (Owner, 2026-11-08)

**D170. Returning a borrow, from Radium.** A function can hand back a borrow by naming where it comes from: `fn warmer [$a: reading, $b: reading] -> Lend from <$a, $b> reading:`. It can come only from a parameter, a named source or `program`, never a local, which ends when the function returns, and never a `Given` parameter, which ends with the function too. The body is checked against the claim, so `-> Lend from $a` returning `Lend $b` is refused. `<$a, $b>` lasts only while both do. At the call, the result's sources become the arguments, and an argument it borrows from has to be a named value, not a fresh one. These are Radium's D49, D127, D130 and D139, which Radium's design claude says are current. A borrow of a plain number is refused, following D123. Lending one item or field, like `Lend $xs[pos: 1]`, is not settled here, and neither are borrows inside structs. The owner said "I love lifetimes" and asked Radium; Claude proposed taking its rules; the owner asked "Where Cyborg lifetimes?" and "Why no named ones", then chose: "yea". (Owner, 2026-11-08)

**D171. Borrows inside structs, from Radium.** A struct names its sources in backticks, declared in `< >`, as in ``struct index <`text`, `table`>:`` with the field ``words: Lend from `text` reading;``. A type fills them with `:` in the declared order, as in ``index<`text`: $article, `table`: $list>``, and leaving one out is refused. Each source is used by a field and has its own name, and `program` is a source that lasts until the program ends. The struct states which source outlives another with ``table`: lasts_at_least `text``, a function states it with `requires $list lasts_at_least $article;`, and every call meets it. A function names a borrow inside a parameter with its own sources: ``fn shown <`t`> [$v: view<`text`: `t`>]``. Writing the same `lasts_at_least` fact twice is refused. These are Radium's D126 to D136, which Radium's design claude says are current. The backticks clash with Markdown, and the owner chose them anyway in Radium. Claude proposed taking them; the owner chose: "yea. Mf, I ain't doing some `'a` shit". (Owner, 2026-11-08)

**D172. Lending a part locks the whole.** `Lend $xs[pos: 1]` and `Lend $r.celsius` work, and the borrow names the whole value it came from, as in `-> Lend from $xs reading`, so all of `$xs` is locked while it's held. Adding to a list can move its items in memory, which would leave a borrow of one pointing at nothing. Borrowing two different fields at once could be allowed later without breaking anything, and isn't settled here. Radium's design claude raised it; Claude proposed it; the owner chose: "Yea". (Owner, 2026-11-08)

**D173. Different fields are separate.** A borrow of one field locks only that field, so `adjust(Lend.mut $r.celsius, Lend.mut $r.time);` works, and so does `change $r.time` while `$r.celsius` is lent. Using all of `$r` still waits for every borrow of its fields, so `Give $r` or `Lend.mut $r` is refused while one is held. The borrow's type still names `$r`, as D172 says. Items in a list stay locked together, since a list can move them and `$xs[pos: $i]` and `$xs[pos: $j]` may be the same one. This narrows D172 for fields. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-08)

**D174. Lending through a stored borrow.** With `let $m: Lend.mut from $r reading = Lend.mut $r;`, `Lend.mut $m` passes it on, and `$m` can't be used until that borrow ends. `Lend $m` passes it on read-only, so `$m` can be read meanwhile but not changed. `Give $m` is refused, since it's only borrowed. A borrow taken through `$m` names `$r`, not `$m`, as in Radium's D142, so nothing holds a borrow of a borrow. Rust does this silently and calls it reborrowing; here it's written, following D3. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-08)

**D175. A parameter isn't typed as a stored borrow.** `fn show [$v: Lend from $r reading]` is refused, because a bare parameter already is a borrow: `fn show [$v: reading]` called with `Lend $view` does the same, and D174 makes that borrow name the original value. The fix says "a bare parameter is already lent, so write `$v: reading`". `Lend.mut from` on a parameter is refused the same way, with `Lent.mut` as the fix. A struct parameter holding a borrow stays as D171 has it. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-08)

**D176. A union case can hold a borrow, like a struct.** A `union.tagged` names its sources in backticks the same way D171 does for structs, as in ``union.tagged lookup <`text`>:`` with the case ``found[item: Lend from `text` reading]``. Every source is used by a case, and a type fills them with `:`. A borrow reached through `match` or the test still names the same source. Claude proposed it, writing the case like a struct field at first, which Claude corrected to D33's form; the owner chose: "yea". (Owner, 2026-11-08)

**D177. A `must_Give` value can be returned or held in a field.** `fn open [$path: str] -> open_file` hands the duty to the caller, who has to `Give` it or `finish` it. A struct holding a `must_Give` field is `must_Give` itself, and says so, as in `struct.must_Give log:`, since D3 rules out anything implicit. A struct that doesn't is refused, with a fix that adds it. `finish` on it still waits for the field to be given away, as D129 says. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-08)

**D178. Kit methods can be `use`d.** `use std::list.%append as %append;` lets a file write `$xs.%append(val: 4);`. The path names the method's type, following D130, and the `%` stays, since it says the method's ownership isn't written at the call. D150 applies, so `as %add` is refused. A method of your own with the same name on the same type in that file is a clash, and refused. This loosens D115 the way every `use` does: the path shows on the `use` line. How a quoted name carries the `%` is not settled here. The owner asked "Can kit methods be `use`'d?"; Claude proposed it; the owner chose: "Yeah". (Owner, 2026-11-08)

**D179. A quoted method name keeps its `%` outside the quotes.** `use std::list.%append as %"push";` gives `$xs.%push(val: 4);`. The `%` warns that ownership isn't written at the call, so a name of your own choosing can't drop it. `as "push"` and `as "%push"` are refused, with `fix: as %"push"`. A limb's short name still drops its `@`, as D155 says, because `@` says where something comes from, which the `use` line shows, and a bare `@name` is a builtin (D56). The owner asked "should quoted still require `%`?" and "should `@` be required even with double quotes?"; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-08)

**D180. A loop says its word.** `for $r in Lend $readings:` reads each item, with the list locked for the loop. `for $r in Lend.mut $readings:` lets you change each item through `$r`. `for $r in Give $readings:` hands you each item, and the list is gone after the loop, which is how you go through `must_Give` items. A bare `for $r in $readings:` is refused, with a fix asking which of the three you meant. The word is on the list, so a list of numbers still takes one, and D123 is about the items. A range is a fresh value, so it takes no word. `$xs.len` stays bare, since it's a field read, not a hand-off. This replaces D2's `for $x in $xs` example. D128 still holds, so `$r` itself can't be pointed at something else. Claude proposed the three words with a bare read; the owner asked "why no `Lend`?" and "what if they forgot to write another one? I mean, we do have `return void`, and `empty`", and chose: "yea". (Owner, 2026-11-08)

**D181. A lent parameter writes `Lent`.** `fn show [$r: Lent reading] -> void:`, so every parameter that isn't a plain number says its word: `Lent`, `Lent.mut` or `Given`. A bare `fn show [$r: reading]` is refused, with `fix: $r: Lent reading`. Plain numbers and `bool` still take none (D123). The call still writes `Lend`. This replaces D2's bare parameter and its refusal of `Lent`, and D175's fix becomes `$v: Lent reading`. It follows the reason for D180: Cyborg writes things out, like `return void` and `empty`, even where a compile error would catch a mistake anyway. The owner asked "What about params?"; Claude proposed it, noting the owner had chosen "no change" for D2; the owner chose: "yea". (Owner, 2026-11-08)

**D182. Back to bare: parameters and loops.** A bare parameter is lent again, as D2 had it, and writing `Lent` is refused. A bare loop reads in place, `for $x in $xs:`, and `for $x in Lend $xs:` is refused, with a fix that drops the word. `Lend.mut` and `Give` on a loop stay, as D180 has them. `Lend $x` at a call stays, because the reader is far from the function there. This replaces D181 and D180's refusal of a bare loop, and D175's fix is `$v: reading` again. The test: a word earns its place when the bare version would compile while being a mistake, as `""` would for `empty` (D103). A wrong bare parameter or loop was already a compile error, and those words were on nearly every function and loop. `return void` stays as it is, `main` only, which the owner kept. The owner asked to "look at the usefulness again, off each explicit thing"; Claude proposed undoing both; the owner called it "the `lend` spam shit" and chose: "yea". (Owner, 2026-11-08)

**D183. Every program needs a manifest.** This replaces D81's exception, where a program using only `std` needed none, so hello world is a source file plus a manifest. The manifest still lists what's installed and which version, and versions live only there. Whether `std`'s version goes in it, and how that works when `std`'s limb part lives in the compiler, are not settled here, and neither is the manifest's format or name. Claude had said `std` ships with the compiler, which the owner never chose; the owner asked "when did I decide it ships with the compiler? And isn't it decided by the manifest?", and chose: "Every program needs a manifest." (Owner, 2026-11-08)

**D184. Numbers can be lent to change.** `fn bump [$n: Lent.mut i32] -> void:` called with `bump(Lend.mut $count);` changes `$count`. `Give`, `Copy` and `Lend` stay refused on numbers and `bool`, as D123 says, since they make no difference there, while `Lend.mut` says the number changes. A plain number parameter can't be changed, following D125, so a function changing its own copy while you expect yours to change can't compile. This makes D173's example, `adjust(Lend.mut $r.celsius, Lend.mut $r.time);`, writable. Oracle claude found that the example couldn't be written; Claude proposed this; the owner asked "would it make a bug there?", and chose: "yea". (Owner, 2026-11-08)

**D185. Moving a field out.** `Give $l.file` takes a field out of something you own, a `Given` parameter or your own `let`. After that, `$l` is incomplete: the only things left are taking out its other fields, or `finish $l`. Reading `$l.file`, lending or giving `$l` whole, and forgetting `$l` without `finish` are refused. A field can't be moved out of something borrowed. This lets a `struct.must_Give` end its duty, which D177 left impossible. Rust calls it a partial move. Oracle claude found the conflict; Claude proposed this; the owner chose: "Yea". (Owner, 2026-11-08)

**D186. A package is what you install.** A package is the unit you install, version and publish, like Rust's crate, and the manifest lists packages. It holds a kit half, Cyborg code reached as `name::`, a limb half, a compiler extension reached as `@name::`, or both, with one name and one version for everything inside, so the halves always fit together. Third-party C it needs goes in its own folder, as D161 allows. `std` is a package with both halves, and is listed in the manifest like any other. The base language and builtins aren't packages. Code doesn't change: the `@` still says which half you reach. Whether your own project is a package, and whether `import std;` brings in both halves, are not settled here. The Java and Go meaning, a folder of code, isn't Cyborg's. Claude proposed `std` following the compiler's version; the owner said "what if a 3rd party wants to create a kit & limb too", then "We need a third category", "I meant the general one. Like, how Rust's crate term", "Define this first", and "Maybe something plain and accurate?"; Claude proposed `package`; the owner chose: "Yea". (Owner, 2026-11-08)

**D187. Import the half you use.** `import std;` brings in a package's kit half, and `import @std;` its limb half, so a file using both writes both. This keeps 9.2 working: a search for `@` finds every file using a compiler extension in its imports. A single line importing both halves is not settled here. Claude proposed it; the owner chose: "Yea", and asked "`import std pkg` does both?" (Owner, 2026-11-08)

**D188. `import pkg std;` imports both halves.** `pkg` names the whole package, as D186 defines it, so `import pkg std;` brings in both `std::` and `@std::`. The word goes before the name, like `@` in `import @csup;` and the word in front of a literal in D165. A search for `@` alone no longer finds every file using a limb, so 9.2's search becomes `@` or `pkg`, and the linter can list each limb a file uses. Claude proposed `import std, @std;`, then `import package std;`; the owner said "`pkg` does shorten a lot, though", asked "What fits more?", and chose: "Yup". (Owner, 2026-11-08)

**D189. Your own project is a package.** It's named in the manifest, so it can be published like any other, and no file says which package it belongs to: there's no line like Go's `package main`, since the manifest and folder already say it, and `fn.main` already marks where a program starts. Your own code is still reached bare, or with `my::` on a clash (D75). The owner asked "Why does Go say Package main? And should we say it too?"; Claude proposed this; the owner chose: "Yea". (Owner, 2026-11-08)

**D190. Cyborg's package manager is CyJet.** It shares only the name with CyborgPL's CyJet. It's Cyborg's own tool, not the same one. The manifest's format and name are not settled here. The owner said "Our PM is also CyJet, btw."; Claude asked whether it's the same tool; the owner chose: "Just name is same." (Owner, 2026-11-08)

**D191. The manifest is `CyJet.zml`.** The Z is a nod to everyone putting a Z in front of their own version of something, like ZJIT and ZGC. It's not XML or any existing `.zml` format. The format inside it is not settled here. Claude found `.zml` used only by a few old, obscure formats; the owner asked "Does `.zml` exist yet?", said "It's a nod every fucking person putting Z Infront of a third party impl", and chose: "That name. Look at fucking ZJIT, ZGC, what the fucking Z is going on". (Owner, 2026-11-08)

**D192. Two hardware milestones.** First, a Cyborg program running on a Raspberry Pi, on Linux on ARM. Second, `std::println` printing over serial on a real Arduino, bare metal with no OS, starting with an ARM board such as the Uno R4, with the 8-bit AVR boards as a stretch goal, since LLVM's AVR support is less mature. CyborgPL has printed with `std` on emulated bare metal, so Cyborg's bar is the real board. Claude proposed them, first with no printing on the Arduino, which the owner corrected: "No printing? Lol, CyborgPL has already achieved emulated bare metal, and printing there with std"; the owner chose: "Yes, please". (Owner, 2026-11-08)

**D193. Linux on ARM runs in a virtual machine.** The first milestone from D192 is a Cyborg program running on Linux on ARM in a virtual machine on the owner's Mac, an Apple M5, instead of on a real Raspberry Pi, since the Pi 5 has reached $385 during a RAM shortage. The second stays on real hardware: `std::println` over serial on an Arduino Uno R4 Minima, about $20, because emulators hide timing and memory problems. This replaces D192's first milestone. The owner said "Or we could just go with bare metal Arduino"; Claude proposed the virtual machine; the owner chose: "Hell yeah". (Owner, 2026-11-08)

**D194. One print: `std::print`, a format print, from CyborgPL.** `std` has one print, `std::print`, and it's always a format print, starting from CyborgPL's `print.format`, which its design claude says is current and tested. Plain text is a format with no holes, so hello world is `std::print("hi{newln}");`. `{}` takes the next value in its readable form, and `{1}`, `{2}` are numbered holes, counting from 1, which can't be mixed with plain ones. Shapes are words: `align: left`, `right` or `center`, `width: 8 cols` with its unit (`cols`, `cols.east_asian`, `graphemes`, `points` or `bytes`), `places: 2`, `base: bin`, `oct`, `hex` or `hex.upper`, `form: science`, and bare `zeros` and `sign`. A newline is `{newln}` and a tab is `{tab}`, inside the text. The text's escapes are `\\ \" \' \r \0 \xHH \uHHHH \UHHHHHHHH`, plus `\{` and `\}` for a brace, and `\n` is refused, with `fix: {newln}`. A value wider than its width prints whole. A literal format is checked when building: the number of holes, a shape that doesn't apply, a width with no unit, and an unknown escape. `std::print.stored` prints every float exactly as stored (D162), on the whole print. `std::println`, `std::println.stored` and every other print are gone, replacing D161's `std::println.stored` and the `std::println` used since D11. Not settled here: how the fill character is written, since Cyborg's backticks are borrow sources (D171), a per-value `form: stored`, stderr, formatting to a `str` without printing, D98's full print with a buffer, and what the output is on bare metal. The owner said "I kinda wanna just have one print in std, and make it cover all cases", "you just fucking say it. Inside", and "I think CyborgPL already has a complete, consistent format print. Ask"; Claude proposed starting from it; the owner chose: "Yeah, and we don't have other prints. to see how it honestly, use `.stored` next to it." (Owner, 2026-11-08)

**D195. `std::print.stderr`.** Printing to stderr is a narrowing, `std::print.stderr("failed{newln}");`, as in CyborgPL. stdout is the default, so `std::print.stdout` stays refused (D50, D51). Whether `.stderr` and `.stored` combine, and in which order, is not settled here. The owner chose: "`.stderr`, obviously". (Owner, 2026-11-08)

**D196. `std` has groups, with `::`.** `std` is grouped, as in `std::io::` and `std::math::sin`, and `use` shortens a long path. `::` is for paths, which never run anything, and a `.` after a function is a narrowing, as in `std::io::fmt_print.stderr`. This replaces D161's flat `std`, which the owner had doubted then: "it might wanna be with `::`". `std::math.sin` stays refused, now with `fix: std::math::sin`. A family is still one level deep. Which groups `std` has, and what the print is called, are not settled here. The owner proposed `std::io::fmt_print`, saying "`::` only appears where it doesn't run anything"; Claude asked whether that replaces the flat `std`; the owner chose: "Yes, because there's `use` to reduce the pain." (Owner, 2026-11-09)

**D197. The print is `std::io::fmt_print`.** It's in `std`'s `io` group, and `fmt_` warns that `{` is special inside the text, so a plain `std::io::fmt_print("{")` is refused, and a brace is written `\{`, with `\` to escape, as D194 has it. Its narrowings stay: `std::io::fmt_print.stored` and `std::io::fmt_print.stderr`. This replaces D194's name, `std::print`. `std::sin` is `std::math::sin` under D196. Claude proposed `std::io::print`; the owner chose: "`fmt_print`. Btw, `\` to esc, `\{`". (Owner, 2026-11-09)

**D198. Importing one thing.** `import std::io::fmt_print;` imports only that one thing, so the file can reach nothing else from `std`, unlike `import std;`. It doesn't give a short name: the file still writes `std::io::fmt_print(…)`, or adds a `use`. Not settled here: whether a file may have both `import std;` and an import of one of its things, and how this works with `@` and `pkg`. The owner proposed it; Claude asked whether it also shortens the name; the owner chose: "It just imports that thing specifically, only. Unlike `import std;`". (Owner, 2026-11-09)

**D199. Unit families live in `std::units`, and an import brings narrowings with it.** A unit family in `std` sits in the `units` group, as in `3600 std::units::time.seconds` and `i64.std::units::time.seconds`, and `use std::units::time.seconds as seconds;` shortens it. Importing a thing brings all its narrowings: `import std::units::time;` gives every unit in the family, since units only make sense with the ones they convert to, and `import std::io::fmt_print;` gives `.stored` and `.stderr`. `fmt_print` isn't a family, but both are spelled `path::thing.narrowing`. This moves the paths from D99 to D112 into D196's groups. The owner asked "what about units?" and "isn't `fmt_print` also a fam?"; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-09)

**D200. `pub`, and private by default.** Anything a package declares is private to that package unless it starts with `pub`, a separate word in front: `pub fn total […] -> i32:`, `pub fn.pure area […]`, `pub struct reading:`, `pub const $max: i32 = 100;`. It's not a narrowing, since it says who can see a thing, not what kind of thing it is, and it applies to everything a package declares. `fn.main` needs no `pub`, since the compiler finds it by its narrowing. Whether a closer repeats `pub`, and finer levels like Rust's `pub(crate)`, are not settled here. The owner asked "is our `fn.main` pub?"; Claude proposed private by default with `fn.pub`; the owner said "Isn't `pub` a separate thing, not a `fn` thing", and chose: "Yeah". (Owner, 2026-11-09)

**D201. A file's rules: `@config`, `@match_config`, `@tighten_config`, `@loosen_config`.** These builtins set the rules the compiler and the linter hold a file to, such as refusing a `let mut` that never changes. Every file starts with one of two: `@config(…);` sets its first rules, or `@match_config(that file);` copies another file's. There's no unsaid default, so a file that wants the compiler's defaults writes `@config(default);`. After that, `@tighten_config(…);` can only make rules stricter and `@loosen_config(…);` only looser, and each refuses the other direction, so a search for `@loosen_config` finds every file that weakens its rules. The manifest only handles packages, so the linter limb is configured this way too, replacing D30's "configured in the project file". In Claude's reading, which the owner didn't object to: a cycle of `@match_config` is refused, a chain is fine, and a matched file that's renamed or missing stops the build. The sketch spelling of a rule, `var::mut::never_changed = refuse`, the list of rules, and where in the file these go are not settled here. The owner proposed all four; Claude proposed tighten-only, and then dropping `@config`; the owner said "`@config` is for setting the first rules in any file, except manifest. That one only handle pkgs", and "You have to say `@config(default);` for default". (Owner, 2026-11-09)

**D202. The linter is part of the compiler.** The official lint rules are built into the compiler, not a limb, so every rule is always available and `@config` can name any of them, with one tool and one way to configure it. A limb can still add rules of its own, named by its path, as in `@tighten_config(@house_style::fn::name_length = refuse);`. This replaces D30's official linter limb. Claude suggested that lint rules sit outside rule 13's Verus proofs, since they're style rather than safety; that is not settled here. The owner asked "would we even want to make the linter a limb anymore"; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-09)

**D203. `decl` and `assign`.** `decl $x: i32;` declares a variable with no value yet, and `assign $x = 1;` gives it its value, once on each path. Every path must `assign` before any read, or it won't compile. After that, `$x` acts like a `let`, and `decl mut $x` lets it change later with `change`. `assign` on something that already has a value is refused, with `fix: change`. It isn't uninitialized memory: nothing can read it empty, so it needs no shield. Rust calls this definite initialization. The owner asked about uninitialized variables and Rust's, then proposed `decl` and `assign`; Claude proposed the rules; the owner chose: "yea". (Owner, 2026-11-09)

**D204. A file's rules come first.** `@config(…);` or `@match_config(…);` is the first line of a file, before any `import` or `use`, and any `@tighten_config` or `@loosen_config` lines follow it straight away, before anything else. This settles where D201's lines go. The standard had put them after the imports, and oracle claude found the conflict with D201's "starts with". The owner chose: "configs are always first". (Owner, 2026-11-09)

**D205. Comments can go anywhere.** A comment can come before a file's rules line, so D204's "first line" means the first line of code, and a `//` comment can trail code on the same line, as in `let $x: i32 = 1; // a note`. A `comment` block's lines still count only when alone on a line (D119), and a `///` doc comment still attaches to what's below it. Oracle claude asked; the owner chose: "Comments can be anywhere, even trailing". (Owner, 2026-11-09)

**D206. A doc comment can't trail code.** `let $t: i32 = 21; /// The temperature.` is refused, with a fix that moves the `///` to its own line above, so docs always sit above what they describe. A trailing `//` stays fine (D205). Oracle claude found the gap; Claude proposed it; the owner chose: "yea". (Owner, 2026-11-09)

**D207. Every error, with cascades found by applying fixes.** Building shows every error, not just the first, in the oracle and the compiler alike. To find knock-on errors, the checker applies the first error's `fix:` in memory and checks again: any later error that disappears was caused by it, and is shown with a `because:` pointing back at it. The errors still there are independent, and the same is done for the next one. An error with no single fix (11.5) can't be applied, so the errors after it stay independent. Because cascades are defined this way rather than guessed, the oracle and the compiler agree byte for byte, and rule 5 needs no exception. Checking again costs time, which the compiler optimizes and the oracle doesn't. The owner asked "Do all errors show at once?", "don't we have `because`?", and "Why wouldn't the oracle show them all? And should we add a cascade removal tech? Like, run the correct version in the oracle before"; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-09)

**D208. Every member line ends with `;`.** A line listing what a struct or union holds ends with `;`, as struct fields already did, so a tagged union's cases are `ok;` and `failed[code: i32];`, and C's untagged union in a shield writes `i: i32;`. Families already did this. 1.3 says `;` ends a statement, and a member line is one. This changes D33's union examples. An outside review, through multifinity, found the mismatch; Claude proposed it; the owner chose: "yea". (Owner, 2026-11-09)

**D209. `fails` goes in the header.** A function's failures are listed after its return type, as in `fn safe_level [$r: reading] -> i32 fails [frozen]:`, so the header says it can fail. Failures stay their own track, not values like Rust's `Result`, so `-> result<i32, frozen>` isn't used, and a comma, `-> i32, frozen`, would read as two return values, as in Go. `labels`, `defaults`, `requires` and `ensures` stay first in the body, in that order. This replaces `fails` in the body from D84 and D114. An outside review, through multifinity, raised it; Claude proposed it; the owner asked "`-> result<i32, frozen>`?" and "`i32, frozen`?", then chose: "yea". (Owner, 2026-11-09)

**D210. Named holes: `{$t}`.** In `std::io::fmt_print`, `{$t}` prints the variable `$t`, the way `{}` prints the next value, as in `fmt_print("temp: {$t} C{newln}");`. The `$` means it can only be a variable, never a shape word like `{zeros}`. Not settled here: shapes on a named hole, a field like `{$r.celsius}`, and mixing named holes with `{}` or numbered ones in one format. An outside review, through multifinity, thought `$` adds nothing without string interpolation; Claude proposed this; the owner chose: "`{}` shit". (Owner, 2026-11-09)

**D211. `.%` lends read-only, `.%%` lends to change.** A method call says how its receiver is lent: `$p.%area();` lends it read-only, and `$room.%%warm_up(by: 3);` lends it to change, so the call alone shows whether the receiver might change. The declaration matches, as in `fn reading.%%warm_up [$r: Lent.mut reading, …]`, and a mismatch is refused. Since `%` only ever stood for `Lend` or `Lend.mut` (D5), nothing is unsaid anymore: `%` and `%%` are short forms of those two words. `use` keeps the mark, as in `use std::list.%%append as %%append;` and `as %%"push"`. This replaces `%` meaning unsaid ownership in D4 and D5. An outside review, through multifinity, pointed out that a `%` call hid whether the receiver changes; the owner asked "Or just drop `%`?", then proposed `%%`; the owner chose: "yea". (Owner, 2026-11-09)

**D212. Constants keep a single `$`.** `const $max: i32 = 100;` is used as `$max`, with no `$$`. Leaving it out hides no mistake, since `change $max` is refused for a `let` too, and in `%%` doubling means more power while a `$$` would mean less. `$$` is also a shell's process ID. The owner proposed `$$` for `const`; Claude proposed keeping `$`; the owner chose: "yea". (Owner, 2026-11-09)

**D213. Spacing is part of the syntax.** No space goes around `.`, `::`, `%`, `%%` or `@`, so `fn . main` and `std :: io` are refused. No space follows `$`, a unary `-`, a `~` on a literal, or an opening `(`, `[` or `<`, and none comes before a closing bracket or a postfix `?`. A binary operator has one space on each side, including word operators like `at_least` and `and`, and `=`, so `1<8` and `$a+$b` are refused, extending D23's rule for `<`. A comma or a label's `:` is followed by one space. Each refusal shows the spaced version as its fix. So every program reads the same, and extra spaces can't be used to line things up. Not settled here: indentation, line length, and the space before an opening `[`, which `fn total [$xs: …]` has and `reading[celsius = 21]` doesn't. The owner said "We should be space sensitive, btw. Because `fn . main` fucking works, an `1<8` shouldn't work either"; Claude proposed the list; the owner chose: "yea". (Owner, 2026-11-09)

**D214. A clashing name says `my::` everywhere, and `main` clashes.** A name that clashes is written `my::x` at its declaration, its closer and every use, as in `fn my::x […] -> i32:` and `end fn my::x`, so a search for `my::x` finds all of them, and the declaration shows the clash. `main` counts as a clashing name, so a plain `fn main` is refused with `fix: fn.main`, and a function really named `main` is `fn my::main`. This replaces D75's bare declaration. The owner pointed out that `fn main` is a real, ordinary function next to `fn.main`, proposed `fn my::main`, then asked "Maybe a clashing name shouldn't stay bare, even at decl?"; Claude proposed the rest; the owner chose: "yea". (Owner, 2026-11-09)

**D215. Checking carries on line by line.** After an error, checking skips only that statement and goes on with the next, so building finds as many errors as it can (D207). A skipped `let` or `decl` still creates its variable, with the type it wrote, as in `let $x: i32 = totl(5);`, which leaves `$x` an `i32`. Cyborg always writes the type in a `let`, so nothing has to be guessed, and D207's fix-and-check-again rule still sorts out knock-on errors. Not settled here: an error in a block's opening line, like `if` or `for`, or in a function's header. Oracle claude asked; Claude proposed checking per item first, then this, when the owner asked "What's better?"; the owner chose: "yea". (Owner, 2026-11-09)

**D216. `:=`: inference, or Precog's pick.** `let $x := …;` and `const $x := …;` leave the type out, and `:=` is the visible sign of that, while a written type uses `=`. It has two modes. Without the Precog limb, or with its rule switched off, it's plain inference: the variable gets exactly the type of the right side, so `let $y := $x + 1;` with `$x` an `i32` makes `$y` an `i32`, and `let $x := 0;` is refused, since a bare literal has no default type (D50), with a fix asking for one. With `import @precog;` and its rule switched on in the file's rules (D201), Precog picks the best type, following CyborgPL: the narrowest plain width that holds every value the variable ever holds, a lone variable at the machine's natural width, and nothing provable is a compile error with no wide fallback. Importing the limb alone never changes what `:=` means. The same program and target always give the same pick, and the build report lists each one. `:=` isn't allowed on parameters, return types or struct fields. Under D215, a broken `:=` line leaves its variable's type unknown. Not settled here: the spelling of Precog's rule, list items and struct fields in best mode, and `:=` on a borrow. This gives `:=` a new meaning, after D83 removed it from `const`. The owner proposed that `:=` comes with the Precog limb and "means pick the best type, not just lazy infer", and asked CyborgPL; Claude proposed CyborgPL's rules with `:=` kept as its own sign; the owner said "without the limb, it would just be lazy infer, right? And maybe a config for precog to be turned on, even with the limb. So, `:=` wouldn't just suddenly change meaning", and chose: "yea". (Owner, 2026-11-09)

**D217. The inline language has its own syntax.** D88's separate language is really a second language, with its own syntax inside its own marked blocks, not Cyborg's syntax with markers on it. Compile-time code runs at a different time, under different rules, so it looks different, and a reader always knows which side they're on. Its style, its blocks' markers, and how it hands code back to Cyborg are not settled here. Claude first sketched Cyborg's syntax with `?` and `bake` on it; the owner asked "Isn't it supposed to be another lang?", Claude asked whether it should be a real second language with its own style, and the owner chose: "yea". (Owner, 2026-11-09)

**D218. No inline language for now: compile-time features instead.** Cyborg drops the separate compile-time language and gives each job its own feature: picking a type by a condition is a `match` on types inside a generic, any number of type parameters is variadic generics, looping over a struct's fields is a built-in loop over a type's fields, and compile-time values stay with `const`, `bake` and `fn.pure.baked` (D91). D89's safety still holds for each feature: checked at the definition, every program it can produce is valid Cyborg, and it always finishes. Generating whole functions or structs waits, and the inline language could come back later as a limb, once the compiler exists. This replaces D88 and D217, and D91's framing of `bake` as a piece of the inline language. Each feature's syntax is not settled here. Claude proposed it, since each feature is much easier to prove safe than a whole second language; the owner had asked "Should we skip the comptime lang, and just make shit a feature?", and chose: "yes". (Owner, 2026-11-09)

**D219. Where a line can break.** A line break inside a statement is allowed only after a comma, after an opening `(`, `[` or `<`, with its closing bracket then on its own line, and after a binary operator, so the line visibly isn't finished. Everywhere else it's refused, with a fix that joins the line, so every block opener and closer stays on one line, and `end` with `fn.main` on the next line is refused. Indentation and line length are still not settled. An outside session in multifinity's chat found that a line break between `end` and `fn.main` was accepted while two spaces weren't; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-09)

**D220. A function's narrowing can't be shortened to its bare name.** `use std::io::fmt_print.stored as stored;` is refused, since `stored(…)` alone reads like it stores something: a function's narrowing only means something next to its function. The fix is `use std::io::fmt_print as fmt_print;`, then `fmt_print.stored(…)`, and a quoted name, as in `as "print_exact"`, is also allowed. Narrowings that aren't a function's, like units, keep D150, so `use std::units::time.seconds as seconds;` stays fine. An outside session in multifinity's chat found that `stored(…)` worked; the owner said "`stored()` is diabolically weird name" and asked "maybe, we refuse the other?"; Claude proposed keeping units; the owner chose: "yea". (Owner, 2026-11-09)

**D221. `.stderr` and `.stored` combine.** One print can go to stderr and print floats exactly as stored. Their order is not settled here. Oracle claude found it open; the owner chose: "`stderr` and `stored` obviously have to be combinable". (Owner, 2026-11-09)

**D222. The compiler is `OKAY\n`, typed `okay`.** A parody of CyborgPL's `KAYO\n`, with nearly the same letters, written the same way: `OKAY\n` as its name and `okay` to type, as in `okay build --mode FinalSafe`. This replaces the placeholder `cyborg build`. A compiler called okay that keeps saying `refused:` is on purpose. Claude pointed out that `ko` would clash with Google's tool of that name; the owner asked "What should the compiler be named? `KO\n`? Or `OKAY\n` (parody to `KAYO\n`)?", and chose: "It's `OKAY\n`, bro". (Owner, 2026-11-09)

**D223. Narrowings go strongest first.** When one thing has two or more narrowings, the one that changes the action most sits next to it, or for safety words, the one that can do more harm. The order is fixed, and the wrong order is refused, with the right spelling as the fix. So a print to stderr that prints floats exactly is `std::io::fmt_print.stored.stderr(…)`, since `.stored` changes what's printed and `.stderr` only where it goes, and `.stderr.stored` is refused. Shapes in a format hole are labels, not narrowings, so they go in any order. Spellings already fixed by earlier decisions, like `fn.pure.baked` and `Lend.mut.held`, stay as they are. The standard keeps a table of narrowing kinds as cases come up, so the order doesn't have to be judged again each time. This settles D221's order. It's CyborgPL's rule, which the owner chose there on 2026-10-01 ("yeah, strongest first"), as its design claude described it; Claude had proposed `.stderr.stored`; the owner chose: "yea". (Owner, 2026-11-09)

**D224. `okay comp` and `okay run`.** `okay comp` compiles and leaves a program, and `okay run` compiles and runs it. The build mode is a flag of its own, as in `okay comp --FinalShit`, replacing `--mode` in D26, and one mode flag is still required every time. `--max_cores=` takes a number, `max` or `min`, and leaving it out means `max`, since a missing core count can't hide a mistake. Flags use underscores, matching Cyborg's names. This replaces `okay build` from D222. Other commands and flags are not settled here. The owner proposed `okay comp/run --FinalShit --max_cores=10/max/min`; Claude proposed `max` as the default; the owner chose: "yup". (Owner, 2026-11-09)

**D225. `extern.c` is gone from the examples.** `@csup::extern.c` came from the owner's example in D93, after Rust's `extern "C"`, but it was never defined, and calls like `c("hi")` meant nothing. The standard now shows a limb's short name with `use @csup::union.untagged as untagged;`, which 9.6 already uses. How a C function is declared and called through `@csup` is not designed, and whether D220 covers a limb function's narrowing waits until it is. Oracle claude asked whether D220 applies to `use @csup::extern.c as c;`; the owner asked "What in the cursed fuck is `extern.c()`?"; Claude proposed dropping it; the owner chose: "yea". (Owner, 2026-11-09)

**D226. Cyborg goes as manual as C, inside shields.** Inside a shield, Cyborg can do what C or Rust's `unsafe` can: raw pointers, with any address and pointer arithmetic; volatile reads and writes that the compiler can't skip or reorder, for hardware registers; exact struct layouts, with field order, packing and alignment fixed; your own allocator, or none at all on bare metal; and inline assembly. The first three join the list of what must live in a shield (D38), and containment (D39) still keeps raw pointers from leaving one. How each is spelled is not settled here. The owner asked "how manual should Cyborg be able to go?"; Claude proposed this; the owner chose: "Yea". (Owner, 2026-11-09)

**D227. `inline` is in the base language, and limbs add to it.** Inline assembly is part of the base language, written bare as `inline.asm`, and still only allowed inside a shield, so a bare-metal program needs no import for it. This replaces `@std::inline.asm` from D28. `inline` is one family, and a limb adds its own inline language to it as `inline.@limb::lang`, as in `inline.@llvm::ir`, so all inline code reads `inline.` first and the `@` still marks the limb (D36). This is the one place a path sits inside a narrowing, rather than first as D196 has it, so that limbs plug into one `inline` instead of each declaring its own. A short name matches the last word, as in `use inline.@llvm::ir as ir;`. How the code inside an inline block is written is not settled here. The owner asked "Should ASM be in base? And then if a limb adds to it, it would be `inline.limb::lang`"; Claude proposed `@llvm::inline.ir`; the owner asked "Shouldn't it be `inline.@llvm::ir`?", said "every limb with inline lang would add their own inline", and chose: "yes". (Owner, 2026-11-09)

**D228. What `CyJet.zml` looks like.** One line per fact, starting with what it is, with no sections, closers, `=` or `;`:

    name weather
    ver 0.1.0
    okay 0.4.0

    need std 0.4.0
    need json 1.2.0
    need csup 0.1.0 with headers

`name` and `ver` are the project's own, since it's a package (D189). `okay` is the compiler version it needs, since a limb plugs into the compiler. Each package it uses gets a `need` line, with its version always written, and `with` adds features. A value is quoted only when it needs to be, as for a space or a `#`. `#` starts a comment. Settings that would need nesting, like per-target options, would use a prefix, as in `target.arduino cores 1`, but none are designed. Not settled here: what features are, the lockfile, and versions as ranges. CyborgPL's CyJet.toml, its fixes, real TOML and Cyborg-style blocks were all shown; the owner asked for "a more no fluff, straight-forward syntax"; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-10)

**D229. The manifest is `CyJet.siml`.** The extension is `.siml`, short for simple, with ML, a markup language, inside it. This replaces D191's `.zml`. The owner chose: "It's called `.siml`. Simple, and also ML inside there". (Owner, 2026-11-10)

**D230. Comments in `CyJet.siml` are `//`.** The same as in Cyborg code (D118), so a project has one comment style, and `#` in a manifest needs no quotes. This replaces `#` in D228. Claude had sketched `#`, after TOML; the owner chose: "`//`". (Owner, 2026-11-10)

**D231. A version is a labelled date and whether it breaks.** A release's version is its date, labelled and biggest first, plus a flag saying whether it breaks anything: `ver 2026y-11m-14d breaks`. The labels mean a day can't be read as a month, and biggest first sorts correctly as plain text, matches ISO dates and follows D223's strongest first. A second release on the same day adds a counter, as in `2026y-11m-14d.2`. The publisher writes the flag, and CyJet checks it against what changed in the `pub` items, refusing a release that changes or removes one but says it doesn't break. A `need` line names a date, as in `need json 2026y-11m-14d`, and CyJet may update it to any later release up to the next one that breaks. The word for "doesn't break" is a placeholder, `same`. This replaces the placeholder `0.1.0` in D228. Claude proposed semantic versioning; the owner asked "Maybe `[date, break or not]`?" and "`ver 14d-11m-2026y breaks`?"; Claude proposed biggest first; the owner chose: "yea". (Owner, 2026-11-10)

**D232. `use` doesn't work on the base language.** `use` only shortens a path with `::` into a kit or limb. Anything in the base language has no path to shorten, so `use inline.asm as asm;` is refused, with a fix that removes the line, since `asm` would look like your own name or an import. Every `inline.…` stays written in full, `inline.@llvm::ir` included, so all inline code reads `inline.` first. This replaces D227's `use inline.@llvm::ir as ir;`. Oracle claude had allowed `use inline.asm as asm;` as a reading; the owner asked "isn't `inline.asm` the base's. Why would you `use` it?"; Claude proposed refusing every base name and every `inline.…`; the owner chose: "yea". (Owner, 2026-11-10)

**D233. How an assembly block looks.** An `inline.asm:` block says in words what goes in and out, then holds raw assembly, one instruction per line, read by LLVM's assembler, and closes with `end inline.asm`:

    inline.asm:
        reads: [x0 = $a, x1 = $b, x2 = $c];
        writes: [x0 => $r];
        clobbers: [];
        madd x0, x0, x1, x2
    end inline.asm

`reads:` puts values into registers before it runs, `writes:` takes registers into variables after, which declares them as a `let` would, and `clobbers:` lists every other register the code changes. It's a statement, not a value. It must be inside a shield, and inside an `on.target` choice that names the chip it's for, so a bare one is refused and there's no `target:` line on the block. `on.target aarch64:` compiles its branch only when building for that chip, with `else:` for the rest, closed with `end on.target`, taken from CyborgPL. The compiler reads the text through LLVM and refuses any register the code writes that isn't in `writes:` or `clobbers:`, and a value narrower than its register needs an `as`, so nothing is widened silently. These close the two gaps CyborgPL's design claude found in CyborgPL's version. Not settled here: a register that's both read and written, lending an address into a register, the narrowings of `on.target` like `.chip` and `.os`, and chips' names. The style is the sketch Radium's design claude wrote for early Cyborg, never ruled on there. The owner asked "Should our ASM be like Radium's or CyborgPL's", then said "I think that I like Radium's more? But we could use CyborgPL's code `on.machine` or whatever it's called"; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-10)

**D234. A type parameter is `?` and a lowercase name.** A generic's type parameters are written like `?item`, declared in `< >` straight after the name with no space, as in `fn largest<?item: ordered> [$a: ?item, $b: ?item] -> ?item:` and `struct pair<?left, ?right>:`. The `?` marks a type not known yet, the way `$` marks a variable, so it can't be mistaken for a real type, and lowercase matches Cyborg's types, like `i32` and `reading`. A one-letter name like `?t` is fine when nothing better fits. Bounds, like `ordered`, are not settled here. Claude proposed it from the placeholder `?T`; the owner chose: "yea". (Owner, 2026-11-10)

**D235. Requirements: interfaces and type sets, from Radium.** What a generic needs from its type parameter is a requirement, not a bound, since "bound" already means a value's min and max (D20). There are two kinds, never mixed. An `interface` lists the operations a type must have, as signatures with no code, and can state laws they obey, include another interface, and leave type slots to fill:

    interface ordered<?self>:
        includes equatable;
        op < [$a: ?self, $b: ?self] -> bool;
        law if $a < $b and $b < $c then $a < $c;
    end interface ordered

A `type_set` is a closed list of types that nothing can join later, as in `type_set whole: i32; i64; end type_set whole`. Requirements combine with `and`, as in `fn smaller<?item: whole and ordered> […]`. A type fits an interface only through an explicit line, `reading satisfies ordered;` (D87), which the compiler checks there, and only the type's package or the interface's package can write it. `reading .|. ordered;` means exactly the same, Radium's deliberate easter egg. A generic body may use on a `?item` only what its requirement lists, so `<` needs an interface with `op <`, and it's checked once, at the definition (D89). This is Radium's D69, D74, D138 and D149, which Radium's design claude says are designed but never built, plus the rule on who writes `satisfies`, which Radium never had. Not settled here: how laws are proven, which is Precog's job, how a type declares its own operators, the spellings of `op`, `law`, `includes` and type slots, whether a parameter with no requirement must write one, and naming a type parameter at a call. CyborgPL's automatic fitting was shown and not taken. The owner said "Try Radium's", then chose: "yea, with the easter egg". (Owner, 2026-11-10)

**D236. A struct can define operators for itself.** A type declares its own operator as a function named by it, as in `op < [$a: reading, $b: reading] -> bool:` … `end op <`, which is how it gets the operations an interface like `ordered` needs (D235). The operator keeps its usual precedence (D116, D117), and applies only where its parameters' types match. An operator can never be defined for, or changed on, Cyborg's built-in types like `i32`. Which operators can be defined, like whether `and` and `or`, which stop early (D124), can be, is not settled here, and neither is a generic operator. Taken from CyborgPL's `op`, as its design claude described it; Claude proposed it; the owner chose: "yea". (Owner, 2026-11-10)

**D237. Which operators a type can define.** A type can define arithmetic, `+`, `-`, `x` and the `div` narrowings, and comparisons, but only `==` and `<` are written: `!=`, `>`, `at_least` and `at_most` follow from them, so they can never disagree. Writing one of those four yourself is refused. `and`, `or` and `not` can't be defined, since they stop early (D124) and an overload couldn't keep that promise, and neither can `=`, `?`, `??` and `as`, which are part of the language, not maths. Not settled here: bit operators, unary `-`, `mod`, and a generic operator. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-10)

**D238. An interface's words are decided.** `op`, `includes` and `law`, written inside an `interface` as in D235's example, are decided, along with `<?self>`. D235's note that their spellings were open came from Radium, where they were provisional, and was Claude's mistake, since the owner had chosen the example as shown. Only how a type slot is written stays open. Oracle claude found that read strictly, every interface would be NotDesignedYet; the owner chose: "yes". (Owner, 2026-11-10)

**D239. Every type parameter writes its requirement, `any` included.** A type parameter that needs nothing says so: `struct pair<?left: any, ?right: any>:`. A body can only store, move, copy and lend an `any` parameter. A bare `?left` is refused, with `fix: ?left: any`, so a reader never wonders whether a requirement was forgotten. This matches the owner's "Don't default to allow all" in CyborgPL. Oracle claude found 5.8's `pair` writing none; Claude proposed allowing it bare; the owner asked "Why doesn't it say so", Claude proposed `any`, and the owner chose: "yea". (Owner, 2026-11-10)

**D240. Only an interface gives operations.** A `type_set` only says which types are allowed, so a body can't use `+` or `<` on a `?item` whose requirement is only a set, even if every type in it has them. It needs an interface too, as in `<?item: whole and addable>`, which also tells a reader what the function does with them. `std` declares which interfaces its own types satisfy, like `addable` and `ordered`, and their names are not settled here. This is CyborgPL's rule, which the owner chose there on 2026-10-28. Oracle claude asked; Claude proposed it; the owner chose: "yea". (Owner, 2026-11-10)

**D241. Where a type parameter comes from.** A call fills each type parameter from its arguments. When they're bare literals, which have no type of their own (D50), the type the result goes into fills it, so in `let $x: i32 = largest(3, 5);`, `?item` is `i32`, since that type is written on the same line. If nothing says a type, as in `let $x := largest(3, 5);`, it's refused, asking for one, as D216 refuses `let $x := 0;`. Naming a type parameter at the call is still not settled. Oracle claude asked; Claude proposed it; the owner chose: "yea". (Owner, 2026-11-10)

**D242. Derived units are declared once, with `per` and `x`.** A unit made of others is declared once, inside a family, as in `motion.speed = length.meters per time.seconds;` and `motion.area = length.meters x length.meters;`. In a unit line, `per` and `x` describe what the unit is made of, and don't calculate anything. The compiler tracks every unit as powers of its bases, so dividing metres by seconds gives metres¹·seconds⁻¹, exactly, with nothing guessed. Maths in the middle of an expression can make any combination without a name, but storing a value needs a declared unit whose powers match, and a mismatch is refused. This replaces D111's `together` lines, one per combination, and keeps their point: you still name a unit before you store it. Not settled here: powers written directly, like a square, and a derived unit's conversions. Multifinity's Claude suggested derived units; Claude proposed this, first with `/` and then `div`; the owner asked "What does `/` mean?" and "Why not `div`?", and chose: "`per`". (Owner, 2026-11-10)

**D243. `any.type` and `any.unit`, and generics over units.** A type parameter that needs nothing writes `any.type`, replacing D239's bare `any`, as in `struct pair<?left: any.type, ?right: any.type>:`. A unit parameter writes `any.unit`, or a family's name for any unit in that family, as in `?u: length`, and goes only where a unit goes, after a number type: `fn average<?u: any.unit> [$xs: std::list<f64.?u>[len.min: 1 items]] -> f64.?u:`. A bare `any` is refused, asking which, since with two kinds a bare one would hide which is meant. A unit parameter where a whole type goes, or a type parameter where a unit goes, is refused. A body can add and compare values of the same `?u`. Storing a product or quotient of a unit parameter is not settled here. Multifinity's Claude suggested generics over units; Claude proposed `?u: unit`; the owner asked "Shouldn't it be `any.unit`?" and "why shouldn't `any` be `.type`?", and chose: "yea". (Owner, 2026-11-10)

**D244. A type parameter takes ownership words, like a struct.** Since a `?item` might be filled by a struct, generic code hands one off with `Give`, `Copy` or `Lend` written, and the word does nothing when it's filled by a plain number. So 5.8's `largest` returns `return Copy $a;`, and a bare `return $a;` on a lent `?item` is refused with a fix offering the word. Treating a type parameter as always copied would silently copy big structs. A `must_Give` type filling a type parameter is not settled here. This was oracle claude's reading, and is CyborgPL's rule; the owner chose: "yea". (Owner, 2026-11-10)

**D245. `std` has angles, and trig asks for radians.** `std::units` has an `angle` family, with radians as the base, degrees as `~0.017453292519943295` radians, and turns as `~6.283185307179586` radians. `std::math::sin` and the other trigonometry functions ask for `f64.std::units::angle.radians`, so a plain number is refused, asking for an angle, and degrees have to be converted with `as`. What comes out of `sin` is a plain number. This makes "radians when you meant degrees" impossible. Not settled here: the rest of the family, and inverse functions like `asin` returning radians. Multifinity's Claude suggested it; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-10)

**D246. `{unit}` prints a value with its unit.** In `std::io::fmt_print`, the bare shape word `unit` prints a value followed by its unit, written the way its own file writes it: with `use std::units::time.seconds as seconds;`, `{unit}` prints `3600 seconds`, and with no `use`, `3600 time.seconds`, with the package path left off. So output matches the code next to it, and the same value can print differently from two files. D122's `as str.with_unit` gives the same text, still as a placeholder spelling. Multifinity's Claude suggested printing with units; Claude proposed it; the owner asked "Where's `use`", and chose: "yea". (Owner, 2026-11-10)

**D247. Named holes take shapes and fields.** A named hole can take shapes after a comma, as in `{$price, places: 2, unit}`, and read a field, as in `{$r.celsius}`, so variables go straight in the text: `fmt_print("{$name}: {$count} at ${$price, places: 2}{newln}");`. A hole only reads a value, never a call or maths, so it never runs anything. Named holes can't be mixed with `{}` or numbered holes in one format. This settles what D210 left open. The owner said "can we put var in the holes, this shit is so fucking hard to read"; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-10)

**D248. `use my::…` shortens your own declarations.** Your own package's path is `my::` (D75), so `use my::temperature.celsius as celsius;` shortens a unit you declared, and the same works for your own types and functions. It's still a path with `::`, so D232 holds: `use` shortens paths and never the base language. D150's matching name and D220's rule on a function's narrowing apply as for any other path. Claude had said a family in your own file couldn't be shortened; the owner said "You're gonna fucking die reading `temperature.celsius` 5000 fucking times"; Claude proposed `my::`; the owner chose: "yea". (Owner, 2026-11-10)

**D249. Points and spans.** In a family where it matters, every value says whether it's a point, a spot on the scale like "it's 20 °C" or "3 PM", or a span, the distance between two spots like "10 °C warmer" or "2 hours": `f64.celsius.point` and `f64.celsius.span`. A bare one is refused there, so nothing is assumed. A point plus a span is a point, a point minus a point is a span, spans add, and a point plus a point is refused. Converting a point applies an offset, so a reading of 0 °C is 273.15 K, while converting a span only scales, so a rise of 10 °C is a rise of 10 K. The families where it matters are ones with an offset conversion, written as a placeholder `temperature.celsius = temperature.kelvin offset ~-273.15;`, and `std`'s `time`, where a moment is a point and a length of time a span, settling D110's moments. Families without the split, like lengths and money, stay as they are. Not settled here: how else a family says it has the split, and the spelling of `offset`. Multifinity's Claude raised offsets; Claude proposed `.at`, then pairs of words; the owner asked "should we make them choose instead of assuming?", and chose: "yea". (Owner, 2026-11-10)

**D250. Prefixes are declared once.** A prefix family lists scales, as in `fam.prefix metric:` with `metric.kilo = 1000;` and `metric.milli = 1 per 1000;`, where `per` keeps the scale exact, unlike `0.001`. A unit opts in with `with`, as in `length.meters: base with metric;`, and the compiler creates `length.kilometers` and `length.millimeters` by joining the prefix to the unit's name, so families stay one level deep (D161). `std` ships `metric`, with kilo, mega, milli and so on, and `binary`, with kibi, mebi and so on, as in `storage.bytes: base with binary;`. `fam.prefix` and `with` are placeholders. Which prefixes `std`'s families list is not settled here. Multifinity's Claude suggested prefixes; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-10)

**D251. Logarithmic units reuse points and spans.** A logarithmic unit like dBm is declared with a log conversion, as in `power.dbm = decibels of 1 power.milliwatts;`, where `decibels of` is a placeholder meaning 10 × log₁₀ of the ratio to that reference. It has D249's split: a level, like "10 dBm", is a point, and a gain, like "adds 3 dB", is a span. So a level plus a gain is a level, which is right, since adding in dB multiplies the power, gains add, and a level plus a level is refused, since two 10 dBm sources together are about 13 dBm, not 20. Combining levels means converting to watts, adding, and converting back. Not settled here: amplitude decibels, which use 20 × log₁₀, other log scales, and the spelling of the conversion. Multifinity's Claude had called them "probably too niche"; Claude repeated that, which the owner had never said; the owner asked "When did I say it was too niche?", Claude proposed this, and the owner chose: "yea". (Owner, 2026-11-10)

**D252. The Arduino milestone starts with a blink.** The real-board milestone from D192 and D193 is an Arduino Uno R4 Minima, an ARM Cortex-M4 with hardware floats and a debug header. Its first step is blinking the built-in LED on pin 13, which needs only a volatile register write inside a shield (D226) and a delay, with no serial or output design. Its second step stays `std::io::fmt_print` over serial. The owner asked "What Arduino to get?" and "Does it come with LED? Maybe make that flash instead"; Claude proposed blinking first and printing second; the owner chose: "yea". (Owner, 2026-11-10)

**D253. The milestone board is the Uno R4 WiFi.** The owner bought an official Arduino Uno R4 WiFi for 1,400 THB, so D252's milestones run on it instead of the Minima. It has the same Renesas RA4M1 ARM chip and the LED on pin 13, so blinking and then printing over serial stay the steps. It has no easy debug header, so debugging is by blink patterns or serial. The owner: "Bought the official Arduino Uno R4 Wifi for 1.4k THB". (Owner, 2026-11-10)

**D254. The point and span split belongs to the whole family.** Every unit in a family that has the split needs `.point` or `.span`, including its base, so `kelvin` and `milliwatts` need one too, not only `celsius` and `dbm`. Adding two readings in kelvin is as meaningless as in Celsius, so the split is about the quantity, not about which unit has the offset. This was oracle claude's reading of D249; Claude recommended keeping it; the owner chose: "yea". (Owner, 2026-11-10)

**D255. Converting units with `as`.** A value changes unit with `as`, as in `$km as f64.length.meters`. A float rounds to the nearest float, like every float operation (D164). An integer converting to a smaller unit, like kilometres to metres, is exact, so plain `as` works. An integer converting to a bigger unit can lose something, since 1500 m is 1.5 km, so it says how with a narrowing, like integer `div` (D48): `as.trunc`, `as.floor`, or `as.exact`, which fails if anything would be lost. A plain `as` there is refused. Points convert with their offset and spans only scale (D249). The full list of narrowings, and conversions between units whose ratio isn't whole, are not settled here. Claude proposed it; the owner asked "Wait, floats don't have to say how to round?", and chose: "yea". (Owner, 2026-11-10)

**D256. The oracle computes maths functions on its own.** The oracle gets correctly rounded results for `std::math::sin` and the rest independently, at very high precision in pure Rust, rounding once at the end, and never uses CORE-MATH or a wrapper of it like the `core-math` crate. That keeps C out of the oracle (rule 1), and keeps the oracle and the compiler from sharing one maths library, whose bugs would then agree (rule 3). The compiler's `std` uses CORE-MATH, which is MIT-licensed, and its answers must match the oracle's bit for bit. Which Rust library or method the oracle uses is its own choice under rule 15. Claude checked CORE-MATH's license and proposed this; the owner chose: "yea". (Owner, 2026-11-10)

**D257. Every float says `.stable`, `.fast` or `.ieee`.** A float type always says which kind it is: `f64.stable` gives the same bits on every machine (D158), `f64.fast` allows merged and reordered maths, and `f64.ieee` allows NaN and infinity. A bare `f64` or `f32` is refused, asking which. The word goes straight after the float type, before a unit or bounds, as in `f64.stable.length.meters` and `f64.stable[min: -40.0, max: 125.0]`. Unlike `Lend` on a loop (D182), the word tells a reader something they wouldn't expect, since floats differ between machines in nearly every other language. C's floats in a shield, like a union's fields, are written `f32.ieee`, since a C value can be NaN. This replaces bare `f64` meaning stable in D158, and `.stable` replaces D158's unnamed default. `.fast` and `.ieee` are still placeholder words. The owner asked "should we let them choose?", said "stable binf isn't really expected, is it?", and chose: "yea. And I mean, I have experience from less naunced AIs saying shit". (Owner, 2026-11-10)

**D258. A type's unit is a label in its brackets.** A unit goes in the type's `[ ]` as `unit:`, next to any bounds, so `i32.time.seconds` becomes `i32[unit: time.seconds]` and `f64.stable.length.meters` becomes `f64.stable[unit: length.meters]`, with bounds as in `f64.stable[unit: length.meters, min: 0.0]`. That keeps the float's kind (D257) and the unit apart, and `[ ]` already describes a type (D23). A point or span stays a narrowing on the unit, as in `[unit: celsius.point]`, and a unit parameter goes the same way, as in `f64.stable[unit: ?u]`. A value still writes its unit after the number, as in `2.5 length.meters`. This replaces the type spelling from D99 and D100. The owner said "`f64.stable.length.meters` looks a bit weird, maybe give it some grouping when needed? `(f64.stable).length.meters`?"; Claude proposed a `unit:` label, since `( )` means running something; the owner asked "What's the `min` for?", and chose: "Yes", then "yup" to the rewrite. (Owner, 2026-11-10)

**D259. A type's labels go unit, then min, then max.** In a type's brackets, `unit:` comes first, then `min:`, then `max:`, as in `f64.stable[unit: length.meters, min: 0.0, max: 100.0]`, and another order is refused with the right one as its fix. That matches labels in calls, which go in declared order by default (5.4), and makes every type read as what it measures, then its range, which is in that unit. A format hole's shapes still go in any order, since they're independent settings. Oracle claude flagged the order as open; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-10)

**D260. A bound on a unit type writes its unit.** A bound in a type with a unit says its unit, the same as any value given to that type, as in `f64.stable[unit: length.meters, min: 0.0 length.meters, max: 1.0 length.kilometers]`. The unit can be any with the same base (D101), and is converted to the type's unit when compiling, while a bound in another base, like seconds on a distance, is refused. For an integer type, that conversion must be exact, so a bound of 1500 metres on `i32[unit: length.kilometers]` is refused rather than rounded. A plain number as a bound is refused, as `let $b: i32[unit: time.seconds] = 3600;` is. Oracle claude found bounds with units still open; Claude proposed plain numbers in the type's unit, then both; the owner said "We have to see first if we can use other units there", then "if we allow both, don't we have to must type? Since, behind `=` must type because of that too", and chose: "yea". (Owner, 2026-11-10)

**D261. `okay` links by itself, like Zig.** Building for any supported target needs nothing else installed: no outside linker, linker script or startup code. `okay` links with lld built in, which Lyftar would embed as a library under rule 1, and ships the startup code and linker scripts for each target it supports, starting with macOS on Apple Silicon, Linux on ARM and the Arduino Uno R4. Cross-compiling is one flag, with a placeholder spelling like `--target=arduino_uno_r4`. Until Lyftar embeds lld, the compiler may call the installed `ld.lld` as a stopgap. Lyftar's design session hasn't agreed to embed lld yet. The owner asked "Does Lyftar support lld?" and said "I kinda want Zig's ease of linking"; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-10)

**D262. Cyborg's licences.** The code is under Apache-2.0 WITH LLVM-exception, so programs compiled with Cyborg owe nothing, even though `std` ends up inside them. The writing, meaning the Markdown files, `DECISIONS.md`, `IMPLEMENTATION.md`, `standard/` and `design/`, is under CC BY 4.0, credited as "Cyborg GitHub repo text: https://github.com/Soclets/Cyborg". These are the same licences as CyborgPL's and Lyftar's, copied from CyborgPL. The repository had been public with no licence since D79. CORE-MATH stays under its own MIT licence, in its own folder (D161). The owner said "Oh fuck, Cyborg isn't licensed", then asked "doesn't CyborgPL (not Cyborg) already use exactly that?"; Claude proposed copying it; the owner chose: "yup". (Owner, 2026-11-10)

**D263. The pitch, third time.** "General sys lang that prioritizes safety & repro over perf." It's the README's first line and the GitHub description. "Repro" means reproducible: the same program gives the same results on every machine, as D158 and D257's stable floats promise. This replaces D157's "loses a little perf for safety". It doesn't mean Cyborg is slow: performance still matters, and the line says only which wins when they conflict. The owner: "Kinda wanna change GH description to "General sys lang that prioritizes safety & repro over perf."", then "yea", and "that doesn't mean shit perf, I actually love perf, I just know it isn't everything". (Owner, 2026-11-10)

**D264. The pitch's wording.** "Syslang that has safety & repro over perf." The owner rewrote D263's line in the README themselves, and it's the GitHub description too. D263's meaning stands: safety and reproducibility win when they conflict with performance, which still matters. The owner: "Bro, I changed README, change GH description yourself". (Owner, 2026-11-10)

**D265. Rules live in the manifest.** The rules the compiler and its linter hold code to are `rule` lines in `CyJet.siml`, one per rule, like the manifest's other facts: `rule var.mut.never_changed refuse`. They apply to every file in the project, and a project with no `rule` lines gets the compiler's defaults. Since every program has a manifest (D183), the rules always live in one known place. An exception for one file names it on the line, as in `rule var.mut.never_changed allow in drivers/raw.cy`, so every loosening is still listed in one place. `@config`, `@match_config`, `@tighten_config` and `@loosen_config` are gone, so code gets straight to the point, and hello world loses its `@config(default);` line. This replaces D201 and D204, and D201's "the manifest only handles packages". A limb's own rules are named by its path as before (D202), and Precog's `:=` mode is switched on with a `rule` line (D216). `rule`, `refuse`, `allow` and `in`, and how a rule's name is spelled, are placeholders. The owner said "I think `@config` is shit, and should be in the manifest instead. So code would get straight to the point."; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-10)

**D266. A new curse line: "GAIM's favorite".** The `FinalShit` curse's random pool (D18) gains the line `GAIM's favorite`, after Google AI Mode's habit of calling C++ "flexible" and "unmatched" while calling safety a cost. The owner: "GAIM likes C++ a lot, calling it flexible, unmatched, and stuff. Might wanna add to the random pool "GAIM's favorite"". (Owner, 2026-11-10)

**D267. A function with no parameters says `[none]`.** `fn.main [none] -> void:`. An empty `[]` is refused, with `fix: [none]`, the same way D103 refuses `[]` for a list. It's `none`, not `empty`, because an empty string or list is a real value with nothing in it yet, while no parameters means there's no value at all. Calls stay `main()`, since `( )` runs something. The owner said "Kinda want empty params to say `[none]`, though C says `void`", then asked whether an empty string or list is the same as no parameters, and chose: "yea". (Owner, 2026-11-10)

**D268. `--FeelLucky` picks a build mode at random.** A nod to Google's "I'm Feeling Lucky": `okay comp --FeelLucky` builds in one of the five modes, `FinalShit` included, chosen at random. It always prints which mode it picked first, as in `feeling lucky: FinalShit`, followed by the curse if that's what came up. It counts as choosing a mode, so D26's "no default build mode" still holds. In a language that prioritizes reproducibility, a flag that makes every build different is the joke. The owner said "`--FeelLucky` is a Google nod, it randomizes the build mode"; Claude proposed the details; the owner chose: "yes". (Owner, 2026-11-10)

**D269. An empty declaration list says `[none]`.** Every bracket list in a declaration that lists nothing says `[none]`, the same as a function with no parameters (D267): an assembly block that clobbers nothing says `clobbers: [none];`, and the same goes for `reads:` and `writes:`. These describe what something takes or touches, rather than being values in memory, so `empty` stays for real values, like lists and strings (D103). A bare `[]` there is refused, with `fix: [none]`. Claude found D233's `clobbers: [];`; Claude recommended `[none]`; the owner chose: "yes". (Owner, 2026-11-10)

**D270. Editor support: highlighting now, `okay lsp` later.** It comes in two levels. First, highlighting and editing basics for VS Code: a TextMate grammar and a language configuration, which are data in JSON, not a program, so they don't bend rule 1. They give `$` variables, `?` type parameters, `@`, `%` and `%%`, `~`, units after numbers, ownership words, `shield` and closers their own looks, plus `//` comments, matching brackets and closers. Second, a language server built into the compiler as `okay lsp`, after the compiler exists, giving errors as you type, go-to-definition, dropdown suggestions, like `str.` offering `utf16`, `div.` offering its narrowings and `std::` offering its groups, and each error's single `fix:` as a one-click quick fix. It comes from the compiler's own checker, so it can't disagree with what compiles. It speaks the Language Server Protocol, an open standard nearly every editor supports, so it doesn't tie Cyborg to VS Code, and each editor only needs a thin adapter. The owner said "Kinda wanna write Cy. Let's design the VS Code support", asked for "code dropdowns", and asked whether building it into `OKAY\n` would "lock us to VS Code, like how you refused inline LLVM IR"; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-10)

**D271. `std` is freestanding, and `os` is a separate package.** `std` is the convenient package, and works anywhere, bare metal included: collections, maths, units, text and formatting. `os` is a separate, plain and portable layer over an operating system, like the terminal, files, processes, arguments, environment and exit codes. Its job is behaving the same on every OS, not being pleasant. A program that uses it says `need os` in its manifest and `import os;` in its files. A bare-metal program needs only `std`. This splits D186's `std` in two. What else goes in `os`, and a bare-metal program's own outputs, like a board's serial port, are not settled here. Multifinity's Claude suggested a freestanding base; the owner said "I was thinking of a freestanding convenience pkg, and a not-so-convenient OS portability pkg", and chose: "Yeah". (Owner, 2026-11-11)

**D272. Printing takes a writer, Zig-style.** `std::io::fmt_print` always takes a writer first, as in `fmt_print(Lend.mut $out, "Hello, World!{newln}");`, where the writer is where output goes: a buffered terminal from `os`, like `os::stdout.buffered(size: 1024 bytes)` or `os::stderr.buffered(…)`, or a board's serial port on bare metal. A buffered writer is `must_Give` (D129), so the program must flush it, as in `os::flush(Give $out);`, and a buffer that silently loses output can't compile. So hello world grows by a writer, a flush and `need os`. `.stderr` is gone, since the writer says where output goes, which replaces D195 and the print row of D223, while `.stored` stays, since it's about formatting. The names `os::stdout.buffered`, `os::stderr.buffered` and `os::flush` are placeholders, and so is how flushing ends the duty. The owner said "I wanna change to a Zig-style print ceremony", chose "Like Zig" over a stream with no buffer, and chose "yup" to Claude's four consequences: the split, a writer on every print, no `.stderr`, and a compile error for forgetting to flush. (Owner, 2026-11-11)

**D273. The terminal writers' names.** `os::stdout` and `os::stderr` are the terminal's two streams, and each must say which kind it is, as D257 does for floats: `os::stdout.buffered(size: 1024 bytes)` writes in chunks and must be flushed, while `os::stdout.unbuffered()` writes each print straight away, can't lose or delay anything, and isn't `must_Give`. A bare `os::stdout()` is refused, asking which. A buffered writer has two operations: `$out.%%flush();` pushes its output now and keeps it going, for something like a progress bar, and `os::done(Give $out);` flushes for the last time and ends its duty, the only way to meet `must_Give`. `done` is a plain function, since a method can't take its receiver (D5). The full words stay, rather than `buff` and `unbuff`, since they're written once per program and "buff" already means something else. This settles D272's placeholder names. Claude proposed them; the owner asked "How do you pick unbuff?" and "Should we shorten to `buff` and `unbuff`?", and chose: "yea". (Owner, 2026-11-11)

**D274. A crash flushes buffered output first.** When a program crashes, including from an unhandled failure (D96) or a failed check, every buffered writer that still holds output is flushed before the crash message, so the last thing printed before a crash always shows. In `FinalShit`, an unhandled failure is UB (D107), so nothing is promised there. If a flush itself fails during a crash, the crash goes on and says so. Oracle claude found it open, and noted Zig loses that output; Claude proposed this; the owner asked "what?", and chose: "yes". (Owner, 2026-11-11)

**D275. `returns` instead of `->`.** A function's header says `returns`, as in `fn half [$n: i32] returns i32 fails [odd]:` and `fn.main [none] returns void:`, so the whole header reads as words, alongside `fails` (D209), the way `at_least` replaces `>=`. `->` is refused, with `fix: returns`. The owner said "Kinda wanna change `->` to `returns`, how does that look", then "I personally read it better, tbh", asked what `Lend from <$a, $b> reading` means, and chose: "yes". (Owner, 2026-11-11)

**D276. `set_to` replaces `=`.** Everywhere Cyborg wrote `=` to give something a value, it writes `set_to`: `let $total: i32 set_to 0;`, `change $total set_to $total + $x;`, a field when building a struct, as in `reading[celsius set_to 21, time set_to 3600 time.seconds]`, a `const`, a default, and a definition, like a unit's conversion or a member of a type family. One word means "this now holds that" everywhere, with nothing to remember about which goes where. A bare `=` is refused, with `fix: set_to`. Comparing with `==` isn't changed here. This replaces `=` in D2, D57, D58, D83 and every decision that used it. The owner had said "I think `=` is a bit weird"; Claude proposed `be` and `to`, then `is` for fields; the owner said "`be` reads weird for fields" and "I don't like `:` there", asked "Why not `set_to` everywhere", and chose: "yes". (Owner, 2026-11-11)

**D277. No type inference in the base language; Precog's pick is `@precog::set_to`.** Every `let` writes its type, so a reader never has to work one out, and D215's recovery after an error can always rely on it. D216's plain inference is gone, and so is `:=`. Precog's pick becomes `let $count @precog::set_to 0;`, which needs `import @precog;` and its `rule` line on (D265), with the `@` and the path marking that a limb chose the type (D36, D56). It can't be shortened to a bare `set_to` with `use`, since that would clash with the base word, so only a quoted short name works. A `let` with no type and no Precog is refused, asking for one. This replaces D216. The owner asked "Or do we fuck type inference? I think we should, tbh. Though, Precog limb could add `@=` or something", then "shouldn't it be `@precog::set_to`?", and chose: "yes". (Owner, 2026-11-11)

**D278. Ordering is written in words: `below` and `above`.** `$a below $b` replaces `$a < $b`, and `$a above $b` replaces `$a > $b`, next to `at_least` and `at_most`, so every ordering says its direction outright. Equality stays as symbols, `==` and `!=`, which programmers read at a glance and which stand out as anchors on a line. So `<` and `>` only ever mean brackets, as in `std::list<i32>`, and the spacing rule that told `$a < $b` apart from `<i32>` (D23) has nothing left to separate. A chain reads `1 below $x below 5`. A type defines `==` and `below` (D237), and `!=`, `above`, `at_least` and `at_most` follow from them, so its operator is `op below`. `$a < $b` and `$a > $b` are refused, with `fix: below` or `fix: above`. Claude proposed `equals` and `not_equal`; the owner said "the `<` `>` directions need a bit to think of", saw every comparison as words and said "everything became one", then read `==` as "equal equal" and `=` as assigning, and chose: "Yes". (Owner, 2026-11-11)

**D279. Each session has a core limit.** On the owner's Mac, which has 10 cores, design claude uses at most 1 core, oracle claude at most 3, and compiler claude at most 3, leaving the rest for the owner. That covers builds and tests, as in `cargo build -j 3`, `CARGO_BUILD_JOBS=3`, and `cargo test -- --test-threads=3`. This replaces rule 9's "no limit". Check claude's limit is not settled here. It's about the sessions' own work, not Cyborg's compiler, whose `--max_cores=` still defaults to `max` for its users (D224). The owner: "It should be: design claude = 1 core, oracle claude = 3 core, compiler claude = 3 core". (Owner, 2026-11-11)

**D280. `Working` keeps every check, even proven ones.** In `Working`, Precog's proofs don't remove checks, so a check Precog wrongly "proved" turns a Precog bug into a loud crash while developing, instead of a silent hole in a release build. Kespar's fuzzer found about a dozen such bugs in its own Precog. Every other mode still drops what Precog proves. This narrows 10.3's "in any mode". The owner asked "Would Precog keep runtime checks in `working`?"; Claude proposed this; the owner chose: "yea". (Owner, 2026-11-11)

**D281. A bare-metal target names the exact board.** A bare-metal build names the board it's for, accurately, since the linker script needs its exact memory map, and its startup code and pins are the board's, not just the chip's. The owner's board is `--target=arduino_uno_r4_wifi`, replacing D261's example `arduino_uno_r4`, since the WiFi and Minima versions differ: on the WiFi, the USB port goes through a separate chip. Chip-level targets, like a plain Cortex-M4, could come later as a building block for new boards. Compiler claude asked; Claude proposed naming the board; the owner chose: "The accurate name, whatever it is, man". (Owner, 2026-11-11)

**D282. A sixth build mode: `Literal`.** `okay comp --Literal` does exactly what the code says. The compiler only does what's needed to make machine code at all, like choosing registers, and never vectorizes, inlines, unrolls, reorders or deletes work the code didn't ask for: if the code doesn't use SIMD, the program doesn't either. It keeps `FinalSafe`'s checks and UB table, so it's about not rewriting code, not about dropping safety. It's for constant-time code like cryptography, honest Purpose benchmarks (D77), embedded timing, and learning. `--FeelLucky` now draws from all six modes. How code asks for SIMD or inlining is not settled here. The owner said "New build mode `Literal`, it does what you say in code literally, if you don't say simd, we don't do simd for you"; Claude proposed `FinalSafe`'s checks and the list of what the compiler won't do; the owner chose: "correct". (Owner, 2026-11-11)

**D283. `=` is back for assigning, and Precog's pick is `@precog::set_to_best`.** Everything D276 changed to `set_to` goes back to `=`: `let $total: i32 = 0;`, `change $total = $total + $x;`, building fields, `const`, defaults and definitions. In a `let`, `set_to` blended into the type name beside it, as in `let mut $out: os::stdout.buffered set_to os::stdout.buffered(…)`, while `=` divides the type from the value at a glance. With equality written `==` and assignment only ever in a `let` or `change` statement, `=` can only mean "assign", which is how the owner reads it. `set_to` is refused, with `fix: =`. Precog's pick gets its own word, `let $count @precog::set_to_best 0;`, replacing D277's `@precog::set_to`. This replaces D276. The owner said "Shit, that's a real problem, you can't just do a clear `=`", "I do read `=` correctly", and chose: "yes, but precog should just get another word. Maybe `@precog::set_to_best`". (Owner, 2026-11-11)

**D284. Sessions keep up with decisions themselves.** Every time oracle claude or compiler claude pulls, which it already does before each push, it reads any new entries in `DECISIONS.md`. A small one in its area, like a renamed word or a new refusal, it just builds. A big one, like a change that touches most of its tests or how a whole part works, it builds after finishing what it's doing, and asks design claude if anything is unclear. So design claude only messages a session when something is urgent, when a decision needs more explaining than `DECISIONS.md` gives, or to wake an idle session. This is rule 16 in `IMPLEMENTATION.md`. The owner said "Or maybe another rule for ease? If someone's in long work, they could look at some new things, and impl it, unless it's big"; Claude proposed the wording; the owner chose: "yes". (Owner, 2026-11-11)

**D285. How many IRs the compiler has is compiler claude's call.** The compiler can have as many intermediate representations of its own as help, and an extra one is fine whenever it makes a check or a proof simpler, since more implementation isn't a cost Cyborg avoids. Whatever the shape, the IRs must serve three things: borrows ending at their last use (D169), which means following every path through a function; Precog dropping checks it proves (D168); and Verus proving the checks (rule 13), which is easier over a small IR. `Literal` (D282) should be able to go to Lyftar without being rewritten. Today the compiler has a syntax tree and a checked program, then Lyftar's IR and LLVM IR. Claude suggested adding a control-flow IR, like Rust's MIR; the owner said "Compiler claudes probably know best, but tbh, more impl shouldn't be a big issue", and chose: "yea". (Owner, 2026-11-11)

**D286. Not built yet is its own error: `NotBuiltYet`.** When the oracle or the compiler reaches something that is decided but that it doesn't handle yet, it says `NotBuiltYet`, never `NotDesignedYet`. `NotDesignedYet` only ever means the design isn't decided (rule 4), so a reader, the owner or check claude can tell a design gap from an implementation gap at a glance. Compiler claude's milestone 1 used `NotDesignedYet` with the why "this compiler doesn't handle it yet"; Claude suggested a separate error, with `NotBuiltYet` as its name; the owner chose: "yes". (Owner, 2026-11-11)

**D287. `->` is back for a function's result.** A function's header says `->` again, as in `fn half [$n: i32] -> i32 fails [odd]:` and `fn.main [none] -> void:`, since a symbol is an anchor the eye can skip straight to, the same reason `==` and `!=` stayed in D278. In a long header, `->` marks exactly where the result starts, while `returns` looked like every other word on the line. `fails` stays a word, and is easy to find since it always comes last. `returns` is refused, with `fix: ->`. This replaces D275. The owner asked "should we change back to `->`?", said "You can skip straight to the symbol", and chose: "yes". (Owner, 2026-11-11)

**D288. `Working` says its optimization with `--opt=`, using LLVM's levels.** `okay run --Working --opt=O0` chooses how much `Working` optimizes, settling D14's "you choose". The levels are LLVM's own, written straight: `O0`, `O1`, `O2`, `O3`, `Os` and `Oz`, so anyone who knows LLVM knows what each does, and Lyftar is the only way to machine code anyway (rule 1). `--opt=` is required with `--Working`, since there's no separate debug mode and `Working` is for both debugging, which wants `O0`, and quick runs, which want `O1` or more, so any default would be wrong for half of them. A bare `--Working` is refused, with `fix: --opt=O0` for debugging or `--opt=O1` for quick runs. `--opt=` with any other mode is refused, since their optimization is fixed. Every check stays at every level (D280). Claude noticed the compiler had picked `O1` itself; the owner said "`Working` is supposed to be selectable" and "it should just say LLVM's opt straight", pointed out "we have no debug. So, they could be either working or debuging", and chose: "yes". (Owner, 2026-11-11)

**D289. An urgent message needs the owner to stop the session.** A message to a session that's busy is queued, and only arrives when its turn ends, which can be a long time in long work. So for something urgent, design claude queues the message and asks the owner to stop that session, so it reads the message straight away. This extends D284 and rule 16. The owner said "msg can't be for urgent either, if they're working, it gets queued. From experience, it's msg to get queue, and then force stop"; Claude proposed the wording; the owner chose: "yes". (Owner, 2026-11-11)

**D290. Sessions in long work keep going.** A finished milestone, a report or a wrap-up isn't a reason for a session in long work to stop: it sends the report and goes straight on. It stops only when it's blocked on a design question with nothing else to work on, when a gap in Lyftar blocks everything, when a test's expected output looks wrong, or at 8:30 PM (rule 12). This is rule 17 in `IMPLEMENTATION.md`, since a session's first prompt is lost to compaction while that file is printed again afterwards. Compiler claude stopped after milestone 3, though its prompt said not to; the owner said "It has already gone through compaction"; Claude proposed the rule; the owner chose: "yes". (Owner, 2026-11-11)

**D291. SIMD is a portable base type: `simd`.** `simd<i32>[lanes: 4]` holds several values, its lanes, and an operator on two of them works on every lane in one instruction, as in `$a + $b`. The compiler picks each CPU's instruction, so the same code works everywhere, the way Zig's `@Vector` and Rust's `std::simd` do. It's in the base language, not a kit or a limb, since a vector is a machine type like `i32`: a kit's `op` could only add one lane at a time, and a limb's `@` would mark it as something outside deciding, when the code is only asking for a vector. This answers half of D282's open question: in `Literal`, SIMD happens exactly where the code writes `simd`. Instructions for one CPU only go in `inline.asm` (D227), with no intrinsics like C's `_mm_add_ps`. Which lane counts are allowed, checks on lanes, float kinds, adding up the lanes, and moving to and from arrays are not settled here. The owner asked "Let's design SIMD?" and "Is it in kit, limb, or base?"; Claude proposed a portable type in the base; the owner chose: "yea". (Owner, 2026-11-11)

**D292. `lanes:` takes any count, or `max`.** A `simd` can have any number of lanes, like `lanes: 3` for an x, y, z point or `lanes: 64`, and the compiler splits it into as many instructions as the CPU needs, padding an odd count, which never changes the answers since lanes are independent. `lanes: max` means the most lanes one instruction works on for the target being built, such as 4 `i32`s on the Mac or 1 on the Cortex-M4. It's a constant for each target, so the same program and target always get the same width, and the build report says which width each target got. `simd` already says "single instruction", so `max` inside it needs no longer name, and it matches `--max_cores=max` (D224), where `max` is also a value. A limit on lanes, if one is ever needed, is only an error, never a value you can write, so `max` can't mean "the most allowed". Claude proposed any count, and the owner asked "Why not 1 & 3?", adding the target's width; Claude proposed `target`, `one_register`, `max_per_instruction` and `max_at_once`, and the owner asked "Why not `max`", said "that's so long", then "doesn't the outside already say `simd`?", and chose: "yes". (Owner, 2026-11-11)

**D293. A lane follows its type's rules exactly.** If any lane of a `simd` overflows or breaks its bound, it's the same failure as for one value of that type, and the message says which lane: `simd<i32[min: 0, max: 100]>` checks every lane's bounds. Checking all the lanes costs about one or two extra vector instructions, not one per lane. Precog drops a check it proves, and each build mode treats these checks the way it treats any other, so SIMD never becomes a way around Cyborg's checks. Wrapping silently, as C and Rust's SIMD do, and saturating aren't defaults: either is something the code writes. Claude proposed this over wrapping and saturating; the owner chose: "yes". (Owner, 2026-11-11)

**D294. Saturating sticks at the type's own bounds.** A saturating operator, where a result that would overflow sticks at the limit instead, uses the type's own bounds, including a written `min:` and `max:`: `$volume plus.saturate 10` on a `u8[min: 0, max: 100]` holding 95 gives 100. On a type with no written bounds, like a plain `u8`, those bounds are the hardware's, so it's one native instruction, like ARM's `uqadd`, and written bounds add a `min` and a `max`, on one value or every lane of a `simd` at once. Sticking at the hardware's limit on a bounded type can't work, since the result wouldn't fit its type, so that's done by saturating in the plain type and then assigning, which checks the bound. The spelling, with `plus.saturate` as a placeholder, is not settled here. The owner said "Let's design satur. now" and asked "How do you do hardware too?"; Claude proposed the type's own bounds; the owner chose: "yea". (Owner, 2026-11-11)

**D295. Wrapping and saturating are `plus`, `minus` and `times` with a narrowing.** `$a plus.wrap $b`, `$a minus.wrap $b` and `$a times.wrap $b` wrap, and `plus.saturate`, `minus.saturate` and `times.saturate` stick at the type's bounds (D294). The operator is written as a word, then how it handles overflow, the same shape as `div.trunc`, since a narrowing goes on a word and `+.wrap` would be hard to read. `saturate` is the term CPU manuals, Rust and ARM use, so it's what people search for. Plain `+`, `-` and `x` stay the checked defaults. `+.wrap` is refused, with `fix: plus.wrap`, and a bare `plus`, `minus` or `times` with `fix: +`, `-` or `x`. This settles D42's placeholder `plus.wrap` and the wrapping forms of `-` and `x`. Whether plain `-` and `x` follow `+`'s overflow row, and the word operators' precedence, are not settled here. Claude proposed the family; the owner chose: "yes". (Owner, 2026-11-11)

**D296. Adding up a `simd`'s lanes uses a fixed pairwise order.** Neighbouring lanes are added in pairs, then the results in pairs, until one is left, with an odd lane out carried up: four lanes add as `(a + b) + (c + d)`, and three as `(a + b) + c`. It's the order SIMD hardware works in naturally, so it's fast, and since the standard fixes it, `.stable` and `.ieee` floats give the same answer on every CPU for the same lane count, even when a narrower CPU splits the vector, and an integer sum overflows at the same place every time. `.fast` floats let the compiler pick any order (D257). With `lanes: max` the lane count changes between targets, and so does the grouping, which choosing `max` accepts. A lane's float kind otherwise follows D293: each lane is a value of its kind. Claude corrected its own earlier claim that `.stable` needed lane-by-lane order; the spelling and the other ways of combining lanes are not settled here. Claude proposed it; the owner chose: "yes". (Owner, 2026-11-11)

**D297. Combining a `simd`'s lanes: six `.%` methods.** `$v.%sum()`, `$v.%product()`, `$v.%min_val()` and `$v.%max_val()`, and on a `simd<bool>`, `$v.%all_true()` and `$v.%any_true()`. They're `.%` since they only read the `simd` (D211). `sum` and `product` use D296's pairwise order and are checked like any other maths (D293). `min_val` and `max_val` use `val`, a word Cyborg already has in labels like `append(val: 4)` and failure data like `negative[val: i32]`, so `max` can't be read as `lanes: max` (D292) or a bound. `all_true` and `any_true` say "true" because `all` is already the fill word in `all 0`. Claude proposed `min` and `max`, then `largest` and `smallest`, and wrongly called `val` only a placeholder; the owner asked "Should it be `%max_val`?" and "Don't we already use `val`", and chose: "yes". (Owner, 2026-11-11)

**D298. `as` converts between an array and a `simd` of the same count.** `$a as simd<i32>[lanes: 4]` turns a `std::array<i32>[len: 4 items]` into a vector, and `as std::array<i32>[len: 4 items]` turns it back. The counts must match, so it can't fail (D255), and a different count is refused when compiling. Walking a longer list or array in chunks is not settled here. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-11)

**D299. Walking in `simd` chunks always says how the leftovers are handled.** When a list or array is walked in chunks of `lanes:`, and its length isn't a multiple of that, the code says which of three ways to handle the rest. Padding fills the last chunk with a value the code gives, like 0 for a sum. Exact requires a multiple, checked when compiling for an array, whose `len` is known, and while running for a list. Leftovers apart hands the leftover items over one at a time after the chunks, as Rust's `chunks_exact` and its remainder do. There's no bare form, since silently dropping the leftovers is a classic SIMD bug that compiles, runs and gives a quietly wrong answer, which is D182's test. The spelling is not settled here. Claude proposed it; the owner chose: "yes". (Owner, 2026-11-11)

**D300. Chunks are `simd_chunks.pad`, `.exact` and `.leftovers`, and leftovers get a section in the loop.** `for $chunk in $xs.%simd_chunks.pad(lanes: 4, pad: 0):` pads the last chunk, `$xs.%simd_chunks.exact(lanes: 4)` requires a multiple, and `$xs.%simd_chunks.leftovers(lanes: 4)` hands the leftover items to a `leftovers $x:` section inside the same loop, after the chunks, before `end for`. The section runs once per leftover item, doesn't run when there are none, and is required: `.leftovers` without it is refused, so the leftovers can't be forgotten, which a separate method and a second loop would allow. A `leftovers` section on any other loop is refused, and `break` stops the whole loop, leftovers included. It's new syntax, which fits since `simd` is in the base language (D291). Claude proposed a section over a second method; the owner chose: "yea". (Owner, 2026-11-11)

**D301. Comparing `simd` values works lane by lane, and `if` on a `simd<bool>` is refused.** All six comparisons, `==`, `!=`, `below`, `above`, `at_least` and `at_most`, give a `simd<bool>` with one answer per lane, as in Zig, where Rust's `==` on vectors gives one `bool` for "all equal". So `==` behaves like the other five, and the code chooses what it wants from the answers: `($a == $b).%all_true()` for every lane, `.%any_true()` for any lane, or the `simd<bool>` itself for which lanes. `if` needs one `bool`, so `if` on a `simd<bool>` is refused, with `fix: .%all_true()` or `.%any_true()`, rather than quietly picking one. Claude proposed it; the owner asked "why not just let them choose?", Claude explained that lane by lane is what lets them choose, and the owner chose: "yea". (Owner, 2026-11-11)

**D302. `.%pick(if_true:, if_false:)` chooses lane by lane.** A `simd<bool>` picks each lane from one of two vectors, as in `($a above $b).%pick(if_true: $a, if_false: $b)` for the larger value in each lane, which is how SIMD code replaces `if`. The labels are `if_true` and `if_false`, since `true`, `false` and `else` are already words. Both sides are worked out in full, for every lane, before picking, since SIMD can't skip a lane, and every lane is checked (D293): `$m.%pick(if_true: $x div.trunc $y, if_false: 0)` fails if any lane of `$y` is 0, even one that `if_false` picks. The fix is making the unused lanes safe first. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-11)

**D303. Writing chunks back is `simd_chunks_mut`.** `$pixels.%%simd_chunks_mut.exact(lanes: 16)` hands out chunks that are written back into the list or array at the end of each round, which is locked for the whole loop. Padding lanes are dropped when writing back, and in a `leftovers` section, a changed item is written back too. `simd_chunks` only reads: its chunks are copies, and changing one is refused, so nobody thinks they changed the array when they didn't. The writing version has its own name, since a method has exactly one mark (D211) and one extra `%` shouldn't silently decide whether an array changes. `mut` is the word Cyborg already uses for "can change", and D211's `%%` says it again: `.%%simd_chunks` is refused with `fix: .%`, and `.%simd_chunks_mut` with `fix: .%%`. Claude first proposed the same name with `%%`; the owner asked "it's the same name? And just one more symbol?"; Claude proposed `simd_chunks_mut`; the owner chose: "yes". (Owner, 2026-11-11)

**D304. `.%shuffle(pos: […])` or `.%shuffle(offset: […])` reorders lanes.** A shuffle lists, for each lane of the result, which lane of the `simd` it comes from, and says `pos:`, counting from 1, or `offset:`, counting from 0, as getting an item does (2.17): `$v.%shuffle(pos: [4, 3, 2, 1])` reverses four lanes. The list is fixed when compiling, since CPUs bake the pattern into the instruction, and a lane that doesn't exist is refused then. The result can have a different count, as in `pos: [1, 1]` giving two lanes, and a lane can repeat. Shuffling by lanes only known while running, a slower table lookup, and names for common shuffles like reversing are not settled here. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-11)

**D305. A shuffle counting from 0 says `offset.lanes:`.** `$v.%shuffle(offset.lanes: [3, 2, 1, 0])`, replacing D304's bare `offset:`, which 2.17 already refuses because it could read as bytes. The unit is `lanes`, matching the type's `lanes:`, rather than `offset.items:` as on lists. A bare `offset:` is refused, with `fix: offset.lanes:`. `pos:` is unchanged. Claude found its own mistake in D304; the owner chose: "yes". (Owner, 2026-11-11)

**D306. `.%lookup` takes lanes worked out while running.** `$table.%lookup(pos: $which)`, or `offset.lanes:`, builds a vector whose lanes come from `$table` at the positions in another `simd`, `$which`, which can change each time, for decoders, hex printing and some cryptography. It has its own name, not `shuffle`, since it's a slower table lookup and can fail: a lane of `$which` pointing past the end is a bounds failure that says which lane (D293), where ARM's `tbl` quietly gives 0 and x86's `pshufb` follows another rule. Bounds on `$which`'s type, like `simd<u8[min: 1, max: 16]>`, prove every lane in range, so the check can be dropped. A `shuffle` list only known while running is refused, with `fix: .%lookup`. Claude proposed it; the owner chose: "yes". (Owner, 2026-11-11)

**D307. Four named shuffles: `reverse`, `rotate`, `join` and `interleave`.** `$v.%reverse()`, `$v.%rotate.left(by: 1)` and `$v.%rotate.right(by: 1)`, `$a.%join(then: $b)`, which puts two vectors end to end, and `$a.%interleave(with: $b)`, which alternates their lanes. `rotate` says its direction as a narrowing, so no one guesses which way a positive number turns, and `by:` is fixed when compiling, like a shuffle list (D304). `join` and `interleave` give one vector with both counts added. Everything else is a `shuffle`, since these four cover most real code and each extra name is one more thing to learn. Claude proposed them; the owner chose: "tea", a typo for "yea". (Owner, 2026-11-11)

**D308. Lyftar embeds clang, and `okay` compiles CORE-MATH itself.** CORE-MATH's C (D161, D256) is compiled by `okay` when a program needs it, for whatever target it's building, through clang embedded in Lyftar the way lld is (D261). So every target `okay` supports gets correctly rounded maths with nothing prebuilt and nothing else installed. Clang is C++, so it goes in Lyftar, as rule 1 says. This is only for the third-party C inside `std` (D161): Cyborg code still can't call users' C, since D225 dropped `extern.c`. `std` ships the small freestanding headers CORE-MATH needs, as `design/core-math-on-cortex-m4.md` lists. Claude recommended compiling CORE-MATH ahead of time and shipping object files instead; the owner chose: "2". (Owner, 2026-11-11)

**D309. Dialects: forked compilers on CyJet, and every manifest says `dialect`.** A Dialect is a compiler forked from, or based on, `OKAY\n`, published on CyJet's registry. It doesn't have to follow any of Cyborg's rules, so code written for one isn't Cyborg and promises none of its checks. CyJet loads a Dialect from the registry the way it fetches a package, and builds the project with it, so `okay` never sees Dialect code. A project says which compiler builds it with a `dialect` line in `CyJet.siml`, as in `dialect mc_mod 2026y-11m-01d`, and plain Cyborg says `dialect original`. Every manifest has the line, so a reader always sees which it is, and `original` is reserved, so no Dialect can take it. A manifest without a `dialect` line is refused, with `fix: dialect original`. How the `dialect` line relates to the `okay` version line, and whether Cyborg packages can depend on Dialect code, are not settled here. Claude suggested keeping Precog a limb, since its dropped checks are part of the real compiler and proven (rule 13, D168, D280). The owner proposed "New "pkg" kind: Dialect. It's doesn't have to follow any rules, just has to fork the compiler", said "CyJet supports loading a Dialect from the registry, and a dialect would have its own compiler but forked/based_on OKAY\n", and chose: "Yes, but you should also say if you use no dialects. Like, `dialect original`, `original` is reserved". (Owner, 2026-11-11)

**D310. `dialect original <version>` replaces the `okay` line.** The manifest's `dialect` line says both which compiler builds the project and which version of it, as in `dialect original 2026y-11m-01d` for `okay` at that version, or `dialect mc_mod 2026y-11m-01d` for a Dialect at its own. That's one line for one fact, the same shape as `need std 2026y-11m-01d`, where D228's `okay` line and D309's `dialect` line had split it in two. An `okay` line is refused, with `fix: dialect original` and its version. Claude proposed merging them; the owner chose: "yes". (Owner, 2026-11-11)

**D311. Plain Cyborg can't use a package written in a Dialect.** A `dialect original` project that needs a package whose own manifest names a Dialect is stopped by CyJet when installing, as in "cool_lib is written in dialect mc_mod, and this project is dialect original". That code needs another compiler, and none of Cyborg's checks cover it, so allowing it would hide unchecked code inside a project that says `original`. Whether a Dialect project can use plain Cyborg packages is up to the Dialect. Claude proposed it; the owner asked "What?", Claude explained it more simply, and the owner chose: "yea". (Owner, 2026-11-11)

**D312. Precog means only the limb.** The name Precog belongs to the `@precog` limb alone: Z3 and CVC5, invariants, lemmas, proof levels and `@precog::set_to_best`. Working out known values and ranges, and dropping checks that can't fail, which D14 and D29 called Precog's layers 1 and 2 in the core, is just `OKAY\n`'s own optimization, like folding constants, and has no name of its own. So "Precog" always means the thing written with an `@`, and a program without `import @precog;` has no Precog. Rules about proven checks, like D280's `Working` keeping them all, cover proofs by either. D168's "Precog dropping checks" in the first compiler is `OKAY\n` dropping them. Claude explained the split as "layers 1 and 2 are in the core"; the owner said "That's confusing, why not just say included is OKAY\n's?", and chose: "yes". (Owner, 2026-11-11)

**D313. Oracle claude writes the random generator and the live comparison, from the standard.** The random program generator and a live runner comparing the oracle and `OKAY\n` are oracle claude's job for now, in `check/`, instead of waiting for check claude. The generator is written from `standard/`, never from `oracle/src`, so what counts as a valid program comes from the docs, the same source check claude would use, and the oracle's own code can't steer it. It builds programs from a syntax tree with one printer and tracks what it doesn't generate (D68). The runner compares both as black boxes on every generated program and every existing test, in every build mode, shrinks each difference to the smallest program that still shows it, and lets the standard decide which is wrong (rule 3). Findings about the compiler go to compiler claude without fixes. Check claude becomes optional, as a second, independent generator later. This replaces D68's rule that the generator can't be the oracle's author's. Claude raised D68's worry that a generator by the oracle's author would share its blind spots; the owner said "isn't programs about what's decided?" and "Just let oracle claude read docs?", and chose: "yes". (Owner, 2026-11-11)

**D314. Raw pointers are `ptr.raw<T>` and `ptr.raw.mut<T>`.** A raw pointer, which only lives in a shield (D38, D226), is `ptr.raw<u32>`, which can only read, or `ptr.raw.mut<u32>`, which can read and write, like Rust's `*const` and `*mut`. Writing through a `ptr.raw` is refused, since even without a safety net that mistake is cheap to catch, and `mut` is Cyborg's word for "can change". `ptr` names the thing first and `raw` narrows it, like `div.trunc`, so `ptr` is a family that other kinds of pointer can join. How pointers are made, read, written and moved is not settled here. Claude proposed `raw<T>` and `raw.mut<T>`; the owner asked "Shouldn't it be ptr.raw or something?", and chose: "yea". (Owner, 2026-11-11)

**D315. Making, reading and writing a raw pointer.** `ptr.raw.mut<u32>.at(address: 0x4004_0000)` makes a pointer from a number, for hardware registers and memory maps. `ptr.raw.mut<i32>.of(Lend.mut $count)` takes a variable's address, with the ownership word saying the access: `Lend` for `ptr.raw` and `Lend.mut` for `ptr.raw.mut`. The borrow checker treats the variable as lent for as long as the pointer exists, and containment (D39) keeps the pointer in its shield, so safe code never sees one dangle. `$p.%read()` reads, and `$p.%%write(val: 1)` writes. Strictly, the pointer itself doesn't change, but `%%` stands for the change to its memory, so a search for `%%` finds every write. Pointer arithmetic is not settled here. Claude proposed the four; the owner chose: "yea". (Owner, 2026-11-11)

**D316. Moving a pointer: `.%moved(offset.items:)` or `.%moved(offset.bytes:)`.** `$reg.%moved(offset.items: 1)` gives a pointer one item on, 4 bytes for a `u32`, and `$reg.%moved(offset.bytes: 2)` gives one 2 bytes on. The unit is always written, as with getting an item (2.17), so C's trap where `p + 1` moves a whole item can't happen silently, and a bare `offset:` is refused, with `fix: offset.items:`. It gives a new pointer and leaves the old one, so moving a variable is `change $p = $p.%moved(…)`, and a negative number moves backwards. Nothing is checked, not even an address wrapping around, since this is shield code, and nothing happens until a read or a write. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-11)

**D317. Volatile is its own pointer kind: `ptr.volatile<T>` and `ptr.volatile.mut<T>`.** Every read and write through a volatile pointer happens, exactly once and in order, and the compiler never skips, merges or reorders them, which hardware registers need: reading a status twice means checking twice, and writing a pin on and then off is two events. It's on the type rather than each access, like Rust's `read_volatile()`, since a register is volatile every time it's touched, and forgetting the mark once is a silent bug that only shows on real hardware. It joins `ptr.raw` in D314's family, with the same `.at`, `.of`, `.%read()`, `.%%write(val:)` and `.%moved`, and `.mut` works the same way. It lives only in a shield (D226). Claude proposed it over marking each access; the owner chose: "yea". (Owner, 2026-11-11)

**D318. When a bare form gets a default.** A bare form is allowed only when its default is both what programmers expect and true to Cyborg's pitch, safety and repro over perf (D264). If no default passes both, the code must choose, with no bare form: floats say `.stable`, `.fast` or `.ieee` (D257), because stable floats aren't what any programmer expects, and unstable ones as the default would cut the pitch. If one default passes both, it's the bare form, and the others are narrowings. This sits beside D182's test, where a word earns its place only when the bare form would hide a mistake. Claude had said a bare form whose worst case is only a cost needs no word; the owner pointed out "`.stable` being assumed is also a cost at worse", gave the real reason: "stable floats aren't really expected by any programmer, and making unstable floats the default would cut our pitches", asked "you never ask whether to record it", and chose: "yea". (Owner, 2026-11-11)

**D319. A bare `struct` keeps the written order; `struct.any_order` and `struct.packed` are narrowings.** A bare `struct` lays its fields out in the written order, padded to their natural alignment, like C and Go, so it matches C structs, hardware and file formats. `struct.any_order` lets the compiler reorder fields to save space, with its choice fixed for each target, and `struct.packed` keeps the written order with no padding, for wire formats and packed registers. Under D318, written order is both what C and Go programmers expect and the safe, reproducible choice, so it's the bare form, while reordering trades safety for size, so it's written. `any_order` says exactly the promise given up, where `arranged` didn't say who arranges. Since a bare `struct` now has an exact layout, D226's list of what must live in a shield needs another look, which is not settled here. Claude proposed letting the compiler arrange a bare `struct`; the owner asked "Do programmers expect the compiler to arrange bare?", "do we usually do that?", "doesn't `.arranged` mean the same", and "why not no bare `struct`?", and chose: "yea". (Owner, 2026-11-11)

**D320. Layouts don't need a shield, and borrowing a packed field is refused.** Exact struct layouts come off D226's list of what must live in a shield (9.8), since a bare `struct` now has one (D319) and a layout isn't dangerous by itself: laying a struct over raw memory already needs a `ptr.raw` or `ptr.volatile`, which are in shields. The one real danger is in `struct.packed`, whose fields can sit at odd addresses. Reading a field by name is a safe unaligned read, since the compiler knows, but a borrow of it would be a misaligned reference that the code using it can't know about, which some CPUs fault on, so `Lend` or `Lend.mut` of a packed field is refused, with a fix to copy it into a variable first, as Rust does. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-11)

**D321. Allocation is explicit: anything that takes heap memory takes an allocator.** As in Zig, and like printing taking a writer (D272), a list that grows, like `std::list<i32>[len.max: inf items]`, is made with an allocator passed in, as in `std::list.new(Lend.mut $alloc)`, so where memory comes from is always said. A list with a finite `len.max` needs none, since its size is known. Running out of memory is a failure, handled like any other, instead of a crash. A program with no allocator, as on bare metal, can't call anything that allocates, which the compiler sees, so no separate no-heap check is needed. A test allocator or an arena can be handed in instead. Writing your own allocator is shield code (D226). The names, like `os::heap` and `std::list.new`, are placeholders, and what an allocator must provide, how a list keeps hold of one, the failure's name, and arenas are not settled here. Claude proposed it over a hidden global allocator; the owner chose: "yea". (Owner, 2026-11-11)

**D322. Every call that allocates is handed the allocator.** A list doesn't hold its allocator, as Zig's unmanaged lists don't: `$xs.%%append(val: 4, alloc: Lend $alloc);`. So a list is plain data with no borrow inside, and it moves, returns and sits in a struct like any other value, where a list holding a borrow would spread lifetimes to every struct with a list field (D170, D171). It also fits D321 literally, since `append` really allocates. The allocator is lent shared, with `Lend`, so many lists can use one, and it changes its own bookkeeping inside its own shield code. The risk is handing a list the wrong allocator, a classic Zig bug, so the list remembers which allocator it uses, as a small ID rather than a borrow, and a different one is a failed check, like a bounds check. Claude first proposed a list remembering its allocator, as noisier to repeat; the owner said "I don't see number 2 being noisy, tbh. But idk"; Claude then found the ownership reason, and the owner chose: "yes". (Owner, 2026-11-11)

**D323. A list that can allocate is `must_Give`.** A list with `len.max: inf items` must be ended, as in `std::list.done(Give $xs, alloc: Lend $alloc);`, which frees its memory with the same allocator, checked by D322's ID. Forgetting it is a compile error (D129), where in Zig forgetting `defer xs.deinit(alloc)` leaks silently. It's on the type, since the compiler can't always know whether a list has allocated yet, and a struct holding one is `must_Give` too (D177). A list with a finite `len.max` needs no allocator (D321), so it isn't `must_Give`. `std::list.done` is a placeholder name, matching `os::done` (D273). Claude proposed it; the owner chose: "yea". (Owner, 2026-11-11)

**D324. An allocator satisfies the `allocator` interface: `allocate`, `resize` and `free`.** Like Zig's `Allocator`, anything that can be passed as `alloc:` satisfies one interface (D235) with three operations: `%allocate`, taking a size and an alignment in bytes, which gives a `ptr.raw.mut<u8>` or fails with `out_of_memory`, the failure lists pass on; `%resize`, which tries to grow or shrink a block where it is and returns whether it could, so a list allocates and copies when it can't; and `%free`, which hands a block back. They're `.%` methods, since allocators are lent shared (D322), and an allocator changes its own bookkeeping inside its shield code. Writing one is shield code, since it deals in raw pointers, and using one only ever shows `alloc: Lend $alloc`. Where the interface lives is not settled here. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-11)

**D325. Checked manual freeing everywhere, with `defer`.** Every value that owns memory is `must_Give` (D129), so nothing is ever freed automatically: growable lists, strings made with `as str`, maps, and any struct or type holding one. You write every free, and the compiler checks every one, so a leak is a compile error, as well as a double free or a use after free, which ownership already caught. That's safer than Rust, where leaks are allowed, and nothing runs that the code doesn't show, unlike Rust's automatic `drop`. It's where D321 and D322 were heading, since a list that doesn't hold its allocator can't be freed automatically anyway. `defer` makes it bearable: `defer std::list.done(Give $xs, alloc: Lend $alloc);` runs when its block ends, however it ends, including `return`, `fail`, `?` and `break`, deferred statements run in reverse order, and a deferred `Give` ends the duty on every path, so it's written once, next to where the value is made. The value stays yours to read and change until then, and giving it away elsewhere is refused. It's what CyborgPL's name meant, "human and machine": you free, and the machine checks, where C trusts blindly and a GC takes memory away. CyborgPL's checked manual memory, generation handles for shared data (CyborgPL's design/checked-memory.md), is a separate idea, not taken here. The owner asked "What if we do checked manual free everywhere? I mean, we have Ownership, but it's gonna become a more manual one. But should still Rust ownership safe", saw it without and with `defer`, asked why CyborgPL is called CyborgPL, and chose: "yes". (Owner, 2026-11-11)

**D326. Strings work like lists: a `len.max` in bytes.** `str[len.max: 32 bytes]` holds its bytes inside the value, like a bounded list: no allocator, nothing to free, and too much text is a failure, like too many items. `str[len.max: inf bytes]` grows on the heap, takes an allocator on each call that grows it (D322), and is `must_Give` (D325), ended with something like `std::text.done(Give $log, alloc: Lend $alloc)`, a placeholder name. The bound is in bytes, not characters, since UTF-8 characters vary in size and memory is counted in bytes. Without this, every `str` would have been a `must_Give` value under D325, even a literal that was never allocated. What a bare `str` means, as for literals and parameters, and how `as str` gets an allocator, are not settled here. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-11)

**D327. A bare `str` is a read-only view of text.** With no `len.max`, `str` is any length, never owns memory, can't change and is never `must_Give`, like Rust's `&str`, Zig's `[]const u8` and C's `const char *`. A literal like `"hi"` is a view that lasts the whole program, and lending a bounded or growable string, as in `load(Lend $log)`, gives one, so a function taking `$path: str` reads every kind of string. A view of another string is a borrow, and follows the stored-borrow rules (D169, D171). Under D318 it's both expected, since that's what a string literal is everywhere, and true to the pitch, since a view can't leak or be freed twice. The three kinds are `str`, a view; `str[len.max: 32 bytes]`, owned inside the value (D326); and `str[len.max: inf bytes]`, owned on the heap and `must_Give`. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-11)

**D328. `as` never allocates, and text from `as` goes into a bounded `str`.** `$n as str[len.max: 20 bytes]` converts a number into text held inside the value, with no allocator. Too much text is a failure, but for numbers the compiler knows the most bytes a type needs, 20 for an `i64` and 11 for an `i32`, so with a big enough bound it can't fail and the check is dropped. A bare `as str` is refused, since a bare `str` is a view (D327) and has to point at text that already exists: the fix gives the bounded form, with the size the type needs, or `Lend $name` when the value is already a string. `as str[len.max: inf bytes]` is refused too, and heap text comes from a function taking an allocator, like `std::text.from($n, alloc: Lend $alloc)`, a placeholder, so `as` stays a plain conversion and every allocation is a visible call (D321). This changes D7's "can allocate memory". Claude proposed it; the owner chose: "yea", and asked "What does bare `as str` do?". (Owner, 2026-11-11)

**D329. Safe and reproducible by default, and as hardcore as you want once you say so.** Cyborg's authors value clear syntax, reproducibility, safety and performance, with safety first when they conflict. Every sharp tool exists, but each has to be asked for by name: build modes like `FinalPerf`, `FinalShit` and `Literal`, `--opt=`, `.fast` floats, explicit `simd`, shields, raw and volatile pointers, Dialects, and tuning for one machine, like profile-guided optimization. Reproducibility is the general rule, and tuning the best performance for a specific machine is fully supported when asked for. Future decisions are judged against this, alongside D318's rule for bare forms. The owner said "Repro is general for us, but we will very much support you, if you want to tune best perf for a specific machine", then "basically Cyborg authors vibe: We like clear syntax, repro, safety, perf (though, safety over). But we will support very hardcore tuning for something"; Claude put it as one sentence; the owner chose: "yea, record it". (Owner, 2026-11-11)

**D330. Lists from an arena still end with `done`.** An arena hands out memory quickly from one big block and frees it all at once, and a list made from it still ends with `std::list.done(Give $xs, alloc: Lend $arena)`, where freeing costs nothing, so every owner follows D325's one rule, whatever allocator it came from. The arena counts what's still alive, and ending it while anything from it is alive is a failed check, so nothing can dangle into a freed arena. The arena's win is speed: allocating is a pointer bump, and freeing is one call at the end. Lists borrowing the arena, needing no `done`, would have saved a line per list but needed a second kind of list type carrying a borrow, which D322 avoided. `std::arena`, `std::arena.new` and `std::arena.done` are placeholder names. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-12)

**D331. `done` ends every owner, and strings use `str.done` and `str.from`.** Every `must_Give` owner ends with a function called `done`: `os::done` for a buffered writer (D273), `std::list.done` for a growable list, `str.done` for a growable string and `std::arena.done` for an arena, so nobody has to remember whether a type says `free`, `close`, `drop` or `deinit`, and `done` fits writers, files and anything else `must_Give`, where `free` only fits memory. Strings' functions are `str.done` and `str.from`, led by the type's own name, with no `std::`, since `str` is a base type, like `ptr.raw.mut<u32>.at` (D315). This settles D323's `std::list.done` and replaces D326 and D328's placeholders `std::text.done` and `std::text.from`. Claude proposed `done` and had invented `std::text`; the owner asked "why `text`?", and chose: "yea". (Owner, 2026-11-12)

**D332. The operating system's allocator is `os::heap`.** `let $heap: os::heap = os::heap();` gives the allocator that takes memory from the operating system, passed as `alloc: Lend $heap`. "The heap" is what programmers already call this memory, and it's in `os` since the memory comes from the operating system, so a bare-metal program, which has no `os`, has no `os::heap` either, and brings its own allocator (D321). This settles D321's placeholder. Claude proposed it over `os::allocator` and `os::memory`; the owner chose: "yea". (Owner, 2026-11-12)

**D333. The interface is `std::allocator`, and the arena is `std::arena`.** D324's interface lives in `std` as `std::allocator`, since `std` is freestanding (D271), so a bare-metal program can write its own allocator against it, and `std::list`, which needs it, is in `std` too. The arena keeps D330's names, `std::arena`, `std::arena.new(size: 1 mib, alloc: Lend $heap)` and `std::arena.done`, and is in `std` rather than `os`, since it only carves up memory it's given, which works anywhere, including on bare metal over a fixed block. This settles the last of D321's open allocator names. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-12)

**D334. Every limb is a native library, trusted when listed, with none privileged.** A limb is the operating system's own library for the machine compiling, a `.dylib`, `.so` or `.dll`, behind a stable C interface, which `OKAY\n` loads while compiling. `std`'s limb half and Precog plug in exactly as anyone else's do, replacing D28's limb half living inside the compiler. There's no sandbox: a limb changes the compiler either way, and Rust's procedural macros and `build.rs` run unsandboxed too, while a native library is about as fast as built-in code. A limb is trusted when the manifest lists it, and an unverified one gets a warning, not a refusal: "a limb runs inside the compiler and can change what your program does. Use it only if you trust where it came from." A limb runs only while compiling, so the program it produces is the same as with built-in code. An unsandboxed limb could make builds differ, by reading a clock or a file, so keeping them reproducible is the limb author's job, a sharp tool the program asks for by listing the limb (D329). This follows CyborgPL's ruling of 2026-10-07, where the owner chose "the OS lib thingies file type" over WebAssembly in a sandbox, and its "warn them, don't refuse". Claude proposed WebAssembly plugins in a sandbox, with `std` and Precog built in; the owner said "Fuck no", then "Ask CyborgPL", and chose: "yea". (Owner, 2026-11-12)

**D335. A limb's syntax is `@`-marked items in a fixed menu of positions.** Everything a limb provides is written with `@` and its path (D22), so it never reserves a plain word and two limbs can't claim the same one. Where an `@limb::thing` may appear is a fixed menu, since the parser has to know the shape before reading it: a type, a call or value, a statement, a block with Cyborg inside, a raw block in another language (`inline.@limb::lang`, D227), in place of `=` in a `let` (as `@precog::set_to_best`, D283), and a `rule` line in the manifest (D265). A limb can't invent grammar anywhere else, and raw text that Cyborg doesn't parse is only allowed inside an `inline.` block, so every file stays readable by any Cyborg reader, editor and tool, whatever limbs it uses. It follows CyborgPL's fixed menu of hooks (2026-10-07), simplified by the `@` mark. What "or just use ours" means for inline languages is not settled here. Claude proposed it; the owner chose: "yea. Though, it can make its own inline, or just use ours". (Owner, 2026-11-12)

**D336. A limb can use Cyborg's families or make its own.** Wherever Cyborg has a family a limb can join, the limb can use ours, joining the base family the way `inline.@mylimb::lang` joins `inline.` (D227), or make its own, a standalone `@mylimb::name` construct doing the same job, like `@inln_stuff::sql: … end @inln_stuff::sql` being an inline language itself. This applies to every position on D335's menu, not just inline languages. Raw text Cyborg doesn't parse is allowed in either kind of raw block, replacing D335's "only inside an `inline.` block", as long as the block ends with its matching `end @limb::name` closer, so an editor or tool that knows nothing about the limb can still skip it. The owner said "use ours is `inline.@mylimb::lang`, make their own is `@inln_stuff::` itself being another inline", then "that applies to other stuff too"; Claude put it as a rule; the owner chose: "yea". (Owner, 2026-11-12)

**D337. A limb hands back Cyborg code, checked as usual, or LLVM IR, listed like shield code.** A limb turns its items into something the compiler builds in one of two ways. Cyborg code is the normal way: `@money::round_half_even($x)` becomes plain Cyborg, and every normal check applies to it, ownership, bounds, units and failures, so a limb can't break safety by accident. LLVM IR, through Lyftar, is for what plain Cyborg can't express, like a GPU kernel, a new calling convention or an instruction Cyborg doesn't have. It's unchecked, like `inline.asm`, so the build report lists it with the limb that produced it, as it lists shields (D34), and a reader can see where unchecked code came from. Nothing is restricted and nothing is hidden (D329). Claude proposed allowing both; the owner chose: "both". (Owner, 2026-11-12)

**D338. What a limb can see and change.** A limb never changes what unmarked code means, so D22's promise holds: a search for `@` finds every compiler extension. Within that, a limb can read the whole checked program, with its types, units and ownership; add errors and warnings anywhere, each with a `fix:`, which only makes programs stricter, never changes what one does; define what its own `@` items mean to any depth, like new types, operators, units and failures, as in an `@sat::i32` whose `+` saturates; and prove that checks can't fail, so they're dropped even in unmarked code, Precog's job, which is the one exception, safe since `Working` keeps every check (D280) and a wrong proof shows up as a crash while developing. Changing what unmarked code means, like making a plain `+` saturate everywhere, is what Dialects are for (D309). It follows CyborgPL's ruling to open the checker and prover deep, with no limb privileged, while keeping Cyborg's `@` mark honest. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-12)

**D339. Secrecy is a `.secret` narrowing on integers and `bool`, in the base language.** `u32.secret` and `bool.secret` mark values that mustn't affect how long the program takes, so constant-time code like cryptography becomes a compiler check instead of a discipline: the compiler refuses anything whose timing or path could depend on a secret, like `if $key == 0:`. It's a narrowing, as `f64.stable` and `ptr.volatile` say what kind of thing something is, and works inside other types, as in `std::array<u8.secret>[len: 32 items]` and `simd<u8.secret>[lanes: 16]`. It's in the base, not a limb, since it's safety the base promises over ordinary code. `Literal` (D282) alone only stopped the optimizer adding branches. What's allowed and refused, overflow on secrets, and how a secret becomes public are not settled here. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-12)

**D340. What's allowed and refused on a secret.** Allowed is what takes the same time for every value on real CPUs: `xor`, `and`, `or` and `not`; `plus`, `minus` and `times`, with overflow left to the next decision; shifts and rotates by a public amount; comparisons, which give a `bool.secret`; and `.%pick(if_true:, if_false:)` on a `bool.secret`, the constant-time choice, which works out both sides and takes no branch, the same method as on SIMD lanes (D302). Refused: `if`, `while` or `match` on a secret, since the path taken leaks it; a secret as a position, as in `$xs[pos: $s]`, since which memory is touched leaks through the cache; shifting by a secret amount, since some CPUs take longer for bigger shifts; `div` and `mod` with a secret on either side, since division takes longer for some values on most CPUs; and passing a secret where a plain value is expected, printing included. A secret mixed with a plain value gives a secret, as in `$key xor 5`, so secrecy spreads to everything it touches and can't be lost by accident. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-12)

**D341. Plain `+`, `-` and `x` are refused on secrets, in favour of `.wrap` or `.saturate`.** A plain `+` checks for overflow, and that check is a branch, so crashing on overflow tells an attacker one bit about the secret. So on a secret, `$a + $b` is refused, with `fix: plus.wrap` or `plus.saturate`, and the word forms from D295 are used: wrapping is what crypto maths does on purpose, modulo 2³², and CPUs saturate without branching, as ARM's `uqadd` does. A bare `+` that quietly wrapped only because its values were secret would break D318, since in Cyborg `+` means checked. Bounds on a secret, as in `u8.secret[max: 100]`, need a check that branches too, so they're only allowed where the compiler or Precog proves the check away, and refused otherwise. This narrows D340's `plus`, `minus` and `times`. Claude proposed it; the owner asked "q" and chose: "yea". (Owner, 2026-11-12)

**D342. A secret becomes public only with `.%reveal()`, inside a shield.** Shields are for code the compiler can't check, where the programmer takes over responsibility and writes down why (D34), and only the programmer knows whether a secret is safe to make public, like ciphertext once encrypted, so `$cipher.%reveal()` gives the plain value and must live in a shield, whose reason says why, as in `reason "ciphertext is safe to publish once encrypted"`. Revealing joins 9.8's list of what must live in a shield, and the build report lists each one. Real crypto has few reveal points, usually one per function that outputs something, and each should be reviewed anyway. Claude first proposed `.%reveal()` anywhere, judging shields only for what can crash; the owner said "First, you have to know what shields are for", and chose: "yea". (Owner, 2026-11-12)

**D343. Tasks are structured: every task finishes before the block that started it ends.** Concurrency and parallelism both start from tasks, and a task is started inside a block and must be joined before that block ends, as Rust's `thread::scope` does, but here it's the only way. A task is `must_Give` (D325), so forgetting to join one is a compile error and no task can leak and keep running in the background. A task can borrow from outside its block, since the block guarantees it ends first. A task's failure comes back when it's joined, and passes up with `?` like any call's. Fire-and-forget tasks that outlive their caller are ruled out, since those are the ones that leak, race at shutdown and swallow errors, and a long-running background job lives in a block as long as it is, like the top of `fn.main`. The spelling, shown as a `parallel:` block with `spawn` and `join`, is not settled here. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-12)

**D344. Concurrency is a limb, `@concur`.** Tasks, spawning, joining and their checks come from a limb, written `@concur::spawn` and `@concur::task<i64>`, not the base language. It's opt-in and `@`-marked, so a search for `@concur` finds every place a program goes parallel, and several models, like structured tasks, actors or async, can exist side by side as their own limbs, none privileged (D334), since concurrency models keep changing over the years. The checks are still real, since a limb can read ownership and add errors anywhere (D338), like refusing two tasks that both `Lend.mut $xs`. D343's structured tasks are `@concur`'s rule. It's still freestanding: a limb only runs while compiling, so what decides whether a program can run on bare metal is where its threads come from, not the limb. It follows CyborgPL, where concurrency was the MCPS limb's. Claude proposed a limb over the base; the owner asked "can you use concur freestanding?", and chose: "limb". (Owner, 2026-11-12)

**D345. Tasks run on an explicit runner.** Every `@concur::spawn` names where its task runs, as in `@concur::spawn(on: Lend $pool) sum(…)`, the way every `append` names its allocator (D322). `std` has a runner interface, like `std::allocator` (D333), so anyone can write a runner, like a board package for a dual-core chip or a test runner, and `std` has a runner that runs tasks one after another on any machine, bare metal included, giving the same answers without the speed-up. `os::threads(count: 8)` gives real threads on an operating system. The thread count is always written, never quietly taken from the machine, so a program behaves the same everywhere unless it chooses otherwise. How to ask for every core is not settled here, since the owner said `os::cores` "doesn't say it uses all cores", and neither are the runners' names. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-12)

**D346. `os::threads(count: max)` asks for a thread for every core.** `max` as a value already means "as many as this machine has" in `--max_cores=max` (D224) and `lanes: max` (D292), so `count: max` follows them. It's written, so a reader sees the count depends on the machine, and a program that wants the same behaviour everywhere writes a number instead (D345). The owner said `os::cores` "doesn't say it uses all cores"; Claude proposed `max`; the owner chose: "Y". (Owner, 2026-11-12)

**D347. Sharing between tasks follows the ownership words.** `Give $xs` hands a value to one task, which owns it from then on; `Lend $xs` lets any number of tasks read it at once; and `Lend.mut $xs` goes to one task only, with nobody else reading it meanwhile, so two tasks both getting `Lend.mut $xs` is refused, the same rule that stops two `Lend.mut` borrows in plain code. Structured tasks (D343) make every borrow into a task end before its block does. Changing a big list in parallel means splitting it into parts that don't overlap, as with `$xs.%%split_mut(at.items: 500)`, a placeholder spelling, giving each task one part, the way different fields are separate (D173). So there are no data races and nothing new to learn, and since each task's result comes back through its own join, in the order the code joins them, results don't depend on which task finishes first. Shared data many tasks change together, which needs a lock, is not settled here. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-12)

**D348. A lock holds its data, unlocks at the end of a block, and can't nest.** `@concur::lock<i64>` holds the data it protects, so there's no way to reach the data without locking, as Rust's `Mutex` does, avoiding C's bug of locking one thing and changing another. A lock is shared with plain `Lend`, so many tasks can hold it, and locking it, as in `@concur::locked $hits as $n: … end @concur::locked`, a placeholder spelling, gives a `Lend.mut` of the inside for that block only. It unlocks when the block ends, however it ends, `fail` and `?` included, so unlocking can't be forgotten. Locking inside another locked block is refused, so deadlocks are impossible, since a deadlock needs a task holding one lock while waiting for another, and a form taking two locks at once in a fixed order can come later. Which task locks first changes between runs, so a lock is where timing can reach results, like the order of a shared list, and under D329 that's a sharp tool asked for by name. Claude proposed it; the owner chose: "yea. Come now". (Owner, 2026-11-12)

**D349. Several locks are taken in one block, always in a fixed order.** `@concur::locked $from, $to as $a, $b: … end @concur::locked` takes every lock it names before the block runs and releases them all when it ends, however it ends. They're taken in an order set by the locks themselves, like their addresses, never by the order written, so two tasks taking the same locks always take them in the same order, and a deadlock, where one task holds A waiting for B while another holds B waiting for A, can't happen. It takes any number of locks. Nesting stays refused (D348), so this block is the only way to hold several, and deadlocks stay impossible. Naming the same lock twice is refused when compiling where it's known, and a failed check otherwise, since a task would wait on itself. The classic case is a transfer changing two accounts at once. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-12)

**D350. Waiting is up to the runner, with no `async` or `await`.** A function that waits, like one reading from the network, takes the runner the way it takes a writer or an allocator, and the runner decides how waiting works: on `os::threads`, a read blocks its thread, and on an event runner, a read that has to wait parks its task and the runner picks up another, so thousands of tasks share a few threads, like Go's goroutines. It's the same code either way, so there's no `async`, no `await`, and no second colour of function, which in other languages spreads `async` up every caller and splits libraries in two. Zig moved to this in 2025. An event runner uses stackful coroutines: each task has its own small stack, and switching saves and restores a few registers, in shield code with `inline.asm`, while the OS (`kqueue`, `epoll`) says when a waiting task can go on. The compiler can work out how much stack a task needs, from the functions it can call (9.10), so only recursive code has to state a size. The event runner's name is not settled here. Claude proposed it; the owner asked "What's `async` and `await`?" and "How?", and chose: "yea, but why don't we just let the writer alloc?". (Owner, 2026-11-12)

**D351. A spawn onto an event runner takes the allocator for its task's stack.** A task's stack is memory, so under D321 and D322 a spawn that allocates one says where it comes from: `@concur::spawn(on: Lend $events, alloc: Lend $arena) handle(Lend $conn);`. The writer picks where stacks live, like the heap, an arena that frees them all when a server shuts down, or a fixed block on bare metal. The compiler still works out the size from the functions the task can call (D350), and recursive code, which it can't measure, writes one, as in `stack: 64 kib`. Running out is an `out_of_memory` failure from the spawn, handled like any other. A spawn onto `os::threads` takes none, since the OS gives a pool's threads their stacks when the pool is made, and D322 only asks for an allocator where something is allocated. The owner asked "why don't we just let the writer alloc?"; Claude proposed the shape; the owner chose: "yea". (Owner, 2026-11-12)

**D352. The writer always writes a task's stack size, and the compiler checks it.** A spawn onto an event runner always says `stack:`, as in `@concur::spawn(on: Lend $events, alloc: Lend $arena, stack: 16 kib) handle(Lend $conn);`, so the writer decides how much memory each task gets and it's visible in the code, like a buffered writer's `size:`. The compiler checks the size instead of choosing it, replacing D350 and D351's "the compiler works out the size": where it can measure what the task can use, a size that's too small is refused, with the number in the fix, as in "handle can use up to 21 kib of stack, and this gives it 16 kib", and for recursive code, which it can't measure, the written size is trusted and running past it is a failed check, a clean stack overflow, never memory corruption. You write it, and the machine checks it. The owner said "I meant alloc the "The compiler still works out the size" part"; Claude proposed the check; the owner chose: "yea". (Owner, 2026-11-12)

**D353. Channels work like lists, move what they send, and make a full channel's sender wait.** A channel, `@concur::channel<job>[len.max: 64 items]`, is a queue between tasks, like Go's `chan`, for pipelines and worker pools without locks. Its capacity is always written, like a list's `len.max` (D13): a finite one needs no allocator, and `len.max: inf items` takes one and is `must_Give`, as lists are (D321, D323). Sending moves the value with `Give`, as in `$jobs.%send(val: Give $j, run: Lend $events)?;`, so a value is only ever owned by one task and channels share data with no races. A full channel makes the sender wait, through its runner (D350), which is backpressure, so a fast producer can't bury a slow worker. When every sender is done, the channel closes, and the receiver's `for $j in Give $jobs.%receive(…)` loop ends. One sender into one receiver keeps its order every run, while with several senders, the order depends on timing, as with locks (D348). The spellings are placeholders. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-12)

**D354. Atomics are strictest by default, with weaker orderings as narrowings.** `@concur::atomic<u64>` is an integer or `bool` many tasks change without a lock, through the CPU's own indivisible instructions, for counters, flags and statistics. It's shared with plain `Lend`, like a lock (D348). `.%add(val: 1)` is checked for overflow like `+`, with `.%add.wrap` and `.%add.saturate` when meant (D295). The bare form uses the strictest memory ordering, sequentially consistent, where every task sees changes in one agreed order, since under D318 it's what people expect and the safe one. The weaker orderings, like `.%load.acquire()` and `.%add.relaxed(val: 1)`, are narrowings for experts, the sharp tool asked for by name (D329), and they can give surprising values but never corrupt memory, so they need no shield. Other operations, like swapping and compare-and-swap, and the names, are not settled here. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-12)

**D355. Tasks need no special block.** D343's sketched `parallel: … end parallel` is dropped: a task can start in any block, like a function body, a loop or an `if`, and still be structured, since it's `must_Give` and so must be joined before that block ends, however it ends, and a task holding a borrow is already made to end before what it borrows by the borrow checker. That's one less construct, with structure coming from rules Cyborg already has (D325, D169). Claude proposed it; the owner asked "What's `ends`", and chose: "yea". (Owner, 2026-11-12)

**D356. `spawn` is a prefix before a call.** `@concur::spawn(on: Lend $pool) sum(Lend $xs)` starts `sum` as a task, like Go's `go f(x)`, so turning a call into a task is only adding a prefix and the call keeps its normal look and labels. The arguments, borrows included, are taken where it's written, and only the body runs later, as a task. That's over a block handing back a value with `instead`, which is heavy for a one-line task, and over the function and its arguments apart, like Zig's `io.async(sum, .{xs})`, which loses the call's look. It adds "a prefix before a call" to D335's menu of limb positions, for any limb. Claude proposed it; the owner chose: "yea". (Owner, 2026-11-12)

**D357. A task ends with `@concur::task.done`, not `join`.** `@concur::task.done(Give $left)` waits for the task to finish, through its runner, gives back its result, passes its failure up with `?`, and ends the task's `must_Give` duty. Every other language calls this `join`, but D331 made `done` the word that ends every owner, so nobody has to remember `free`, `close`, `drop` or `deinit`, and a task is an owner, so `join` would be the one exception D331 exists to prevent. It reads as "the task is done, give me what it made". Claude proposed it; the owner chose: "yea". (Owner, 2026-11-12)

**D358. Changing versions are narrowed with `.mut`, not `_mut`.** Splitting a list into parts tasks can change is `$xs.%%split.mut(at.items: 500)`, and D303's writing chunks become `$pixels.%%simd_chunks.mut.exact(lanes: 16)`, replacing D303's `simd_chunks_mut`. Cyborg narrows with a dot everywhere, as in `ptr.raw.mut`, `Lend.mut` and `div.trunc`, and a `_mut` glued onto a name was the odd one out. D303's worry still holds: the changing version looks different from the reading one by a whole word, not one extra `%`, and D211's `%%` says it again. `at.items:` says its unit, like `offset.items:`. Claude had proposed `_mut` in D303 and `split_mut` here; the owner asked "shouldn't it be `.mut`?", and chose: "yea". (Owner, 2026-11-12)

**D359. `@concur::locked` and `os::events` keep their names.** Locking stays `@concur::locked $from, $to as $a, $b: … end @concur::locked`, a block with Cyborg inside from the limb menu (D335), where "locked … as" reads as what it does, and the event runner stays `os::events`, next to `os::threads`, named for what it runs on, the operating system's events, like `kqueue` and `epoll`. This settles D348 and D350's placeholders, so `@concur`'s spellings are all decided except atomics' other operations. Claude proposed keeping them; the owner chose: "yea". (Owner, 2026-11-12)

**D360. Atomics' other operations: `swap`, `compare_swap`, and the rest of the `add` family.** `$flag.%swap(val: 1)` sets a new value and gives back the old one, in one indivisible step. `$owner.%compare_swap(expect: 0, val: $me)` sets the new value only if the current one is still `expect`, the building block for lock-free code, and if something else got there first, it fails with `not_expected[val: …]`, carrying what's there now, so a lost race uses Cyborg's normal failures and can't be ignored, where most languages return a boolean or a pair. `.%sub`, `.%and`, `.%or` and `.%xor` work like `.%add` (D354), with `.wrap` or `.saturate` where overflow applies, and every operation takes D354's ordering narrowings, like `.%swap.acquire(…)`. Only the strong `compare_swap` exists: the weak one some CPUs offer, which can fail for no reason, could be a narrowing later. Claude proposed it; the owner chose: "yea", and asked "What's `%and`?". (Owner, 2026-11-12)

**D361. Bit operators are the logic operators narrowed with `.bit`.** `$a and.bit $b`, `$a or.bit $b`, `$a xor.bit $b` and `not.bit $a` replace D49's `bit_and`, `bit_or`, `bit_xor` and `bit_not`, since Cyborg narrows with a dot, the same shape as `plus.wrap` (D295), and `bit_` was glued on before that was settled. Plain `and`, `or` and `not` stay `bool` logic, so `.bit` is the visible difference. There's no plain `xor`, since `!=` does that for bools, and a bare `xor` is refused, with `fix: xor.bit`. `shl` and `shr` stay, having no plain version to narrow. This also corrects two of Claude's mistakes: D340's secret operations, which said `xor`, `and`, `or` and `not` where it meant bits, are `and.bit`, `or.bit`, `xor.bit` and `not.bit` on secret integers, as in `$key xor.bit $nonce`, and D360's atomics are `.%and.bit`, `.%or.bit` and `.%xor.bit`. Whether `and` and `or` on a `bool.secret` short-circuit, which would leak, is not settled here. The owner asked "Why is its name the same, then" and "why the fuck not `.bit` after `and`?", and chose: "yea". (Owner, 2026-11-12)

**D362. Plain `and` and `or` are refused on secrets, in favour of `and.both` and `or.both`.** Plain `and` and `or` stop early (3.6), so `$a and $b` only runs `$b` when `$a` is true, and that skip is a branch that leaks `$a`. On a `bool.secret`, they're refused, with `fix: and.both` or `or.both`, D124's narrowing that always runs both sides, the same way plain `+` is refused for `plus.wrap` (D341). `not` is fine, having one side and no branch. So plain `and` keeps its meaning everywhere (D318), and a secret just can't use the form that skips. `.both` is still D124's placeholder name. Claude found it fixing D340 for D361; Claude proposed it; the owner chose: "yea". (Owner, 2026-11-12)

**D363. Profile-guided optimization: `--profile.record` and `--profile.use=`, with a committed profile file.** `okay comp --FinalPerf --profile.record server.cy` builds with counters, running it on real work writes a profile file, and `--profile.use=server.profile` builds tuned for that work: hot calls inlined, the common path laid out straight. The profile is a file committed next to the code, so the same code and the same profile always give the same program and repro survives (D329). It's about the program and its work, not the machine, so it helps on any CPU for that target, goes stale as the code changes, and should come from real work, not tiny tests. It changes only speed, never what the program does, works in every optimizing mode, and is refused in `Literal` (D282) and `--Working --opt=O0`. The build report says which profile was used, so a benchmark can't quietly be a tuned build (D78). Claude proposed it; the owner asked "Is this like hardware specific? Or program specific", and chose: "yea". (Owner, 2026-11-12)

**D364. Ceil splits into `Ceil.source`, `Ceil.machine` and `Ceil.program`, with no bare `Ceil`.** D77's Ceil, "anything allowed", mixed two things that make a number less comparable. `Ceil.source` is anything in the code, built for the target's baseline CPU with no profile, a number anyone can reproduce on any machine of that target. `Ceil.machine` adds tuning for one CPU, which only holds on that chip, and `Ceil.program` adds a profile (D363), which only holds for that workload, with `Ceil.machine.program` for both. There's no bare `Ceil`, so every number says which kind of fastest it is, and under D78 a result reads like "fib, `Ceil.machine` (apple_m5), `FinalPerf`". Other languages follow the same rules (D77), so Rust's `-C target-cpu=native` row and C's `-march=native` row are `Ceil.machine`, and `bench/`'s Ceil rows were relabelled to match. The owner proposed "a new type, `ceil.machine`, and `ceil.program`", said "Benchmarks", and chose: "yes, and maybe no bare `Ceil`? So, that becomes `Ceil.source`?". (Owner, 2026-11-12)

**D365. Tuning for one machine is `--machine.cpu=` and `--machine.gpu=`.** Without tuning, a build uses the target's baseline, so it runs on every chip of that target. `--machine.` tunes for one machine's parts, matching `Ceil.machine` (D364): `.cpu` for its processor and `.gpu` for its graphics chip, and other parts, like an AI accelerator, can join as narrowings without a new flag. An exact name, like `--machine.cpu=apple_m5`, is reproducible, as a board's is (D281), and `native` means the machine building, which the build report records as the name it meant. A tuned program checks the machine when it starts and stops with an error naming any missing instruction, instead of crashing midway. The base handles `.cpu`, and `.gpu` goes to whichever limb gives the program its GPU code (D337, D338), since Cyborg has no GPU code of its own, and it's refused when no such limb is used. It's refused with `Literal` too. Claude proposed `--cpu=`; the owner said "but GPUs exist"; Claude proposed the family; the owner chose: "yea". (Owner, 2026-11-12)

**D366. A map's types are `key.type:` and `val.type:`.** `std::map.hash<key.type: str, val.type: i32>`, so a bare `val:` is always a value, as in `append(val: 4)` and `put(key: "a", val: 1)`, and `val.type:` is only ever its type, the way a list's length type is already `len.type: u32` (2.10). D24's `std::map<key: str, val: i32>` was the one place a label carried a type, which read like a call handing over values. Fields, as in a struct's `celsius: i32;` or a failure's data, keep `name: type`, since they're declared with `:` and filled with `=`. `std::map<key: str, val: i32>` is refused, with `fix: key.type:` and `val.type:`. This settles D24's placeholder label names. Chapter 8's example failure data was renamed from `val` to `got` alongside, to show it's the writer's own name. MultClaude compared Kotlin's `val`; the owner said "we have two meanings of `val`", asked "why doesn't `val` in a map mean the same? It's the same fucking spelling", and chose: "yes". (Owner, 2026-11-12)

**D367. A view of a literal needs nothing; a view of another string writes its source when stored or returned.** A literal like `"kitchen"` is built into the program and never goes away, so a bare `str` holding one is stored in struct fields and returned freely, with no source written, like Rust's `&'static str`. A view of another string is a borrow: passing one into a function, as in `fn load [$path: str]`, needs nothing, since its borrow ends with the call, but storing or returning one writes where it's from, as in `fn first_word [$text: str] -> Lend from $text str:`, following D170 and D171, which is exactly where a dangling string could happen. A field holding text that isn't a literal stores owned text instead, as `str[len.max: 64 bytes]`. This settles how D327's "a view of another string is a borrow" applies, which oracle claude asked. Claude proposed it; the owner asked "what", and chose: "yea". (Owner, 2026-11-12)

**D368. Copying a view needs no word, and a `let mut` view can point at other text.** A bare `str` is a pointer and a length to read-only text (D327), so copying one can't change or free anything, and `let $k: str = $s;` needs no `Copy`, like a plain number (6.6): under D182's test, a word there would hide no mistake. Owned strings, with a `len.max`, still take `Copy` or `Give`, so 6.7's example now copies a `str[len.max: 32 bytes]`. A view's text can't change, but `let mut $s: str = "a";` then `change $s = "b";` points the variable at other text, so 2.9's "can't change" means the text. Oracle claude asked both; Claude proposed them; the owner chose: "yea". (Owner, 2026-11-12)

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
- **Placeholders.** The repository's name and the license are not chosen.
