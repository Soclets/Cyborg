# 10. Build modes

| Section | What | In short |
|---|---|---|
| [10.1](#101-the-modes) | The six modes | One table |
| [10.2](#102-no-default) | `--mode` | Always say which |
| [10.3](#103-what-gets-checked) | Checks | What stays in each mode |
| [10.4](#104-what-happens-when-something-goes-wrong) | UB | **The one table of what's UB where** |
| [10.5](#105-overflow) | Overflow | A plain `+` |
| [10.6](#106-assuming) | Assuming | `FinalPerf` and `FinalShit` treat promises as facts |
| [10.7](#107-finalshit) | `FinalShit` | Drops the compile-time safety rules |
| [10.8](#108-the-curse) | The curse | What `FinalShit` prints |
| [10.9](#109-benchmarks) | Benchmarks | Three kinds, in every mode |

## 10.1 The modes

**Rule:**

| Mode | Optimization | Runtime checks | For |
|---|---|---|---|
| `Working` | You choose | All kept, no matter the cost | Developing and debugging |
| `FinalSafe` | Max | Cheap checks only | Safe releases |
| `FinalPerf` | Max | None | Trust your tests |
| `FinalShit` | Max | None | A joke, with more UB possible than C |
| `FinalSmall` | For size | Whatever makes it smaller | Size only, with UB and bad speed allowed |
| `Literal` | Only what's needed to make machine code | Same as `FinalSafe` | Exactly what the code says, with no SIMD, inlining, unrolling, reordering or deleting it didn't ask for |

**From:** D14, D26, D282.

## 10.2 No default

**Rule:** there's no default build mode. You always say which, as its own flag. The compiler is `OKAY\n`, typed `okay`: `okay comp` compiles, and `okay run` compiles and runs. `--max_cores=` takes a number, `max` or `min`, and is `max` when left out. `okay` links by itself, with nothing else installed, and cross-compiles with one flag, as in `--target=arduino_uno_r4_wifi`. A bare-metal target names the exact board, since its memory map, startup code and pins are the board's.

```
okay comp --FinalSafe
```

`--FeelLucky` picks one of the six modes at random, `FinalShit` included, and always prints which one first:

```
$ okay comp --FeelLucky
feeling lucky: FinalShit
```

It counts as choosing a mode. In a language about reproducibility, a build that's different every time is the joke.

**Refused:** `okay comp` with no mode flag, and `--mode FinalSafe`, with `fix: --FinalSafe`.

**From:** D26, D222, D224, D261, D268, D281.

## 10.3 What gets checked

**Rule:** anything Precog proves can drop its check, in every mode except `Working`, which keeps every check, proven or not, so a wrong proof shows up as a crash while developing.

| Check | `Working` | `FinalSafe` | `FinalPerf` | `FinalShit` | `FinalSmall` | `Literal` |
|---|---|---|---|---|---|---|
| Overflow, bounds, division by zero | Kept | Kept, as traps | Gone | Gone | If smaller | Kept, as traps |
| A shield's exit (`ensures`) | Kept | Kept, even if expensive | Gone | Gone | Gone if smaller | Kept, even if expensive |
| A shield's entry (`requires`) | Kept | Follows the mode | Gone | Gone | If smaller | Follows the mode |
| Other contracts | Kept | Kept, unless marked | Gone | Gone | If smaller | Kept, unless marked |
| `if $n: T:` on a plain value | Kept | Kept | Kept | Kept | Kept | Kept |

A check in `Working` is kept no matter the cost. `.only_in_working` on `requires` or `ensures` marks a contract as checked in `Working` only:

```
requires.only_in_working is_sorted(Lend $xs);
```

**From:** D14, D20, D26, D37, D39, D105, D106, D280.

## 10.4 What happens when something goes wrong

**Rule:** this is the one table of what's UB where.

| What goes wrong | `Working` | `FinalSafe` | `FinalPerf` | `FinalShit` | `FinalSmall` | `Literal` |
|---|---|---|---|---|---|---|
| A plain `+` overflows | Trap | Trap | Wraps | UB | Whatever's smaller | Trap |
| A value is out of bounds | Crash | Trap | UB | UB | UB allowed | Trap |
| Division by zero | Trap | Trap | UB | UB | UB allowed | Trap |
| A contract is false | Crash | See [10.3](#103-what-gets-checked) | UB | UB | UB allowed | See [10.3](#103-what-gets-checked) |
| A float becomes NaN or infinity | Crash | Trap | UB | UB | UB allowed | Trap |
| A failure nothing handles | Crash | Crash | Crash | UB | Crash | Crash |
| A compile-time safety rule is broken | Refused | Refused | Refused | UB | Refused | Refused |

The `FinalPerf` entries follow from D14 having no checks there; the owner's answer was "you test your shit throughly first".

**From:** D14, D20, D26, D40, D41, D42, D96, D107, D158.

## 10.5 Overflow

**Rule:** see the first row of [10.4](#104-what-happens-when-something-goes-wrong). `FinalPerf` wraps because a plain `+` isn't a promise you wrote. To always wrap, use [3.8](3-operators.md#38-wrapping-pluswrap).

Whether `-` and `x` follow the same table isn't stated ([12.3](12-not-decided.md#123-operators-and-conversion)).

**From:** D42.

## 10.6 Assuming

**Rule:** `FinalPerf` and `FinalShit` treat contracts and bounds as facts for the optimizer, so it can delete code based on them. `FinalPerf` does this because it assumes you've tested thoroughly. It isn't comedic: it keeps every compile-time safety rule, and has no curse.

```
fn clamp [$n: i32[min: 0, max: 100]] returns i32:
    if $n at_least 101:
        return 100;
    end if
    return $n;
end fn clamp
```

In both modes, the compiler can delete that `if`.

**From:** D40, D41.

## 10.7 `FinalShit`

**Rule:** `FinalShit` assumes nothing bad can happen, and keeps only what it needs to generate code: syntax, known names and types that fit. So a program can build only in `FinalShit`.

Everything else that exists to stop you doing something dumb is gone: ownership words, borrows, `must_Give`, `%`, `~`, `*`, units, containment, what must live in a shield, bounds on literals, and covering every case in a `match`, where a missed case is UB. It assumes nothing ever fails, so an unhandled failure is UB, not a crash, and `?` and `otherwise:` may get deleted.

**From:** D39, D40, D107.

## 10.8 The curse

**Rule:** building in `FinalShit` prints this. It's exempt from the one-fix rule in [11.5](11-errors.md#115-fixes).

```
$ okay comp --FinalShit

warning: you chose FinalShit
what:  every contract is now a fucking promise nobody checked
why:   you asked for more UB than C, you absolute legend
fix:   use FinalSafe, or pray
your shit better not be committed

Compiling... ☠️
Done. Good fucking luck.
```

The line after `fix:` is picked at random on each build, and this changes only the printed message, never the binary:

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
GAIM's favorite
```

**From:** D15, D16, D17, D18, D19, D266.

## 10.9 Benchmarks

**Rule:** every benchmark runs in every build mode, in each of three kinds. Any number quoted says its kind and its mode.

| Kind | What it's for | How it's written |
|---|---|---|
| Purpose | Isolates one thing, and the result says what | As hard as it can, for that one thing |
| Normal | How Cyborg does in everyday code | No `inline` code, shields or hand-tuning |
| Ceil | How fast Cyborg can possibly go | Anything allowed |

Other languages' versions follow the same rules.

**From:** D32, D77, D78.
