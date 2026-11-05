# 10. Build modes

| Section | What | In short |
|---|---|---|
| [10.1](#101-the-modes) | The five modes | One table |
| [10.2](#102-no-default) | `--mode` | Always say which |
| [10.3](#103-what-gets-checked) | Checks | What stays in each mode |
| [10.4](#104-what-happens-when-something-goes-wrong) | UB | **The one table of what's UB where** |
| [10.5](#105-overflow) | Overflow | A plain `+` |
| [10.6](#106-assuming) | Assuming | `FinalPerf` and `FinalShit` treat promises as facts |
| [10.7](#107-finalshit) | `FinalShit` | Drops the compile-time safety rules |
| [10.8](#108-the-curse) | The curse | What `FinalShit` prints |
| [10.9](#109-benchmarks) | Benchmarks | Published ones use `FinalSafe` |

## 10.1 The modes

**Rule:**

| Mode | Optimization | Runtime checks | For |
|---|---|---|---|
| `Working` | You choose | All kept, no matter the cost | Developing and debugging |
| `FinalSafe` | Max | Cheap checks only | Safe releases |
| `FinalPerf` | Max | None | Trust your tests |
| `FinalShit` | Max | None | A joke, with more UB possible than C |
| `FinalSmall` | For size | Whatever makes it smaller | Size only, with UB and bad speed allowed |

**From:** D14, D26.

## 10.2 No default

**Rule:** there's no default build mode. You always say which.

```
cyborg build --mode FinalSafe
```

**From:** D26.

## 10.3 What gets checked

**Rule:** anything Precog proves can drop its check, in any mode.

| Check | `Working` | `FinalSafe` | `FinalPerf` | `FinalShit` | `FinalSmall` |
|---|---|---|---|---|---|
| Overflow, bounds, division by zero | Kept | Kept, as traps | Gone | Gone | If smaller |
| A shield's exit (`postcond`) | Kept | Kept, even if expensive | Gone | Gone | Gone if smaller |
| A shield's entry (`precond`) | Kept | Follows the mode | Gone | Gone | If smaller |
| Other contracts | Kept | Not decided if expensive | Gone | Gone | If smaller |
| `if $n: T:` on a plain value | Kept | Kept | Kept | Kept | Kept |

A check in `Working` is kept no matter the cost. Whether `FinalSafe` keeps an expensive contract outside a shield's exit isn't decided ([12.7](12-not-decided.md#127-build-modes)).

**From:** D14, D20, D26, D37, D39.

## 10.4 What happens when something goes wrong

**Rule:** this is the one table of what's UB where.

| What goes wrong | `Working` | `FinalSafe` | `FinalPerf` | `FinalShit` | `FinalSmall` |
|---|---|---|---|---|---|
| A plain `+` overflows | Trap | Trap | Wraps | UB | Whatever's smaller |
| A value is out of bounds | Crash | Trap | UB | UB | UB allowed |
| Division by zero | Trap | Trap | UB | UB | UB allowed |
| A contract is false | Crash | See [10.3](#103-what-gets-checked) | UB | UB | UB allowed |
| A compile-time safety rule is broken | Refused | Refused | Refused | UB | Refused |

The `FinalPerf` entries follow from D14 having no checks there; the owner's answer was "you test your shit throughly first".

**From:** D14, D20, D26, D40, D41, D42.

## 10.5 Overflow

**Rule:** see the first row of [10.4](#104-what-happens-when-something-goes-wrong). `FinalPerf` wraps because a plain `+` isn't a promise you wrote. To always wrap, use [3.8](3-operators.md#38-wrapping-pluswrap).

Whether `-` and `x` follow the same table isn't stated ([12.3](12-not-decided.md#123-operators-and-conversion)).

**From:** D42.

## 10.6 Assuming

**Rule:** `FinalPerf` and `FinalShit` treat contracts and bounds as facts for the optimizer, so it can delete code based on them. `FinalPerf` does this because it assumes you've tested thoroughly. It isn't comedic: it keeps every compile-time safety rule, and has no curse.

```
fn clamp [$n: i32[min: 0, max: 100]] -> i32:
    if $n at_least 101:
        return 100;
    end if
    return $n;
end fn clamp
```

In both modes, the compiler can delete that `if`.

**From:** D40, D41.

## 10.7 `FinalShit`

**Rule:** `FinalShit` assumes nothing bad can happen. It drops every compile-time safety rule: ownership, borrows, `must_Give`, containment, what must live in a shield, and bounds on literals. So a program can build only in `FinalShit`.

The compiler still refuses what it can't compile: bad syntax, unknown names and types that don't fit.

**From:** D39, D40.

## 10.8 The curse

**Rule:** building in `FinalShit` prints this. It's exempt from the one-fix rule in [11.5](11-errors.md#115-fixes).

```
$ cyborg build --mode FinalShit

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
```

**From:** D15, D16, D17, D18, D19.

## 10.9 Benchmarks

**Rule:** a published Cyborg benchmark is built in `FinalSafe`, and says so.

**From:** D14, D32.
