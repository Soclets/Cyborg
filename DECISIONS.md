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

## Open

- **Whether the test in D9 works on a plain variable.** Claude suggested refusing it, since `if $n: bool:` on a value
  whose type is already known is always true or always false. Not chosen yet.
- **How much of Radium's narrowing rule applies.** A bare name as the default, and spelled-out defaults refused. Not
  chosen yet.
- **Radium's open ownership questions, inherited with D2.** Lending through a stored `Lend.mut`; how a `must_Give` duty
  ends in the function that receives it; a `must_Give` value in a field or returned; a parameter whose type is a stored
  borrow; a `union.tagged` case holding a borrow; going through items that are structs, unions or arrays.
- **Type bounds.** The owner suggested `i32[min: …, max: …]`. How a value gets into a bounded type is not chosen yet. Claude
  suggested `$n as i32[min: 0, max: 100]`, which crashes visibly; the owner pointed out a plain `=` could do the same
  check silently. An `if` test stays in every build mode, since it is the program's own logic.
- **Placeholders.** The repository's name and visibility, the license and the file extension are not chosen.
