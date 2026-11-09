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
