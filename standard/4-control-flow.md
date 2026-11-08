# 4. Control flow

| Section | What | In short |
|---|---|---|
| [4.1](#41-if) | `if`, `else if`, `else` | Runs a block when true |
| [4.2](#42-the-test-if-n-t) | `if $n: T:` | Tests a type, and trusts it inside |
| [4.3](#43-match) | `match` | Must cover every case |
| [4.4](#44-for) | `for` | Ends by going through something |
| [4.5](#45-ranges) | `range[from: , to: ]` | A sequence of numbers |
| [4.6](#46-while) | `while` | Ends when a `bool` stops holding |
| [4.7](#47-forever) | `forever` | Has no end, unless you `break` |
| [4.8](#48-break) | `break` | Stops a loop |

Each loop says how it ends. There is no `continue`: wrap the work in an `if` instead.

## 4.1 `if`

**Rule:** `if` runs its block when the condition is true.

```
if $t at_least 10:
    std::println("big total: ", $t as str);
end if
```

`else if` and `else:` continue it, and one `end if` closes the whole chain. The test in [4.2](#42-the-test-if-n-t) works in `else if` too.

```
if $n > 100:
    std::println("huge");
else if $n > 0:
    std::println("big");
else:
    std::println("small");
end if
```

**From:** D126, the sketch.

## 4.2 The test: `if $n: T:`

**Rule:** `if $s: status.failed:` reads as "if `$s` is that kind, do this". Inside the block, the compiler treats `$s` as that kind. This is called flow typing.

It works on a union's case, and on a bound for a plain integer.

```
if $s: status.failed:
    std::println("failed: ", $s.code as str);
end if

if $n: i32[min: 0, max: 100]:
    use_score($n);
end if
```

Outside the block, `$s.code` is a compile error.

Changing `$s` inside the block, or lending it with `Lend.mut`, is allowed, and the compiler's trust ends there:

```
if $s: status.failed:
    std::println($s.code as str);
    change $s = status.ok;
end if
```

**Refused:** reading `$s.code` after that `change`, with `because:` pointing at it.

The test is the program's own logic, so on a plain integer it stays in every build mode. Adding a case to a union later doesn't break this test: it keeps compiling and skips the new case.

**From:** D9, D20, D33, D104.

## 4.3 `match`

**Rule:** `match` handles a union's cases, and must cover every one. Adding a case to a union later forces every `match` to handle it.

```
match $s:
    case status.ok:
        std::println("worked");
    case status.failed:
        std::println("failed");
end match
```

Naming a case lets its arm read its data:

```
match $s:
    case status.failed $f:
        std::println($f.code as str);
    case status.ok:
        std::println("worked");
end match
```

**Refused:** a `match` that misses a case.

**From:** D9, D33, D139.

## 4.4 `for`

**Rule:** `for` goes through a collection or range. A bare collection is read in place, `Lend.mut` lets each item change, and `Give` hands you each item. A range takes no word, since it's a fresh value. Its own value can't change inside.

```
for $x in $xs:
    change $sum = $sum + $x;
end for

for $r in Lend.mut $readings:
    change $r.celsius = $r.celsius + 1;
end for

for $f in Give $files:
    close(Give $f);
end for
```

Bare, the list is locked for the loop. With `Lend.mut`, each item can change through its name. With `Give`, each item is yours, and the list is gone after the loop.

**Refused:** `for $x in Lend $xs:`, with a fix that drops the word.

**From:** D2, D128, D180, D182, the sketch.

## 4.5 Ranges

**Rule:** a range is a sequence of numbers. `from:` is required, `to:` includes the end, `before:` stops before it, and `step_by:` comes after them, where a negative one counts down. The order is fixed.

```
for $i in range[from: 1, to: 5]:
for $i in range[from: 1, before: 5]:
for $i in range[from: 0, to: 10, step_by: 2]:
for $i in range[from: 5, to: 1, step_by: -1]:
```

A range also takes numbers in `items`, so `range[from: 1, to: $xs.len]` goes through every position.

When the numbers are only known while running, a range that can't reach its end runs zero rounds, with no error, so an empty list just skips the loop. A step of 0 is a failure. The linter can flag a range that looks wrong.

`step_by:` takes a literal or any integer, and doesn't have to match the range's type, so `range[from: $xs.len, to: 1, step_by: -1]` counts down a length.

**Refused:** a literal range that can never reach its end, as in `range[from: 5, to: 1]`, and labels out of order.

Inside the loop, the compiler knows the value is within its range, and skips any check it can see will pass, such as `$xs[pos: $i]` when going up to `$xs.len`. If the loop could make that untrue, the check stays.

**From:** D128, D142, D145, D146, D147.

## 4.6 `while`

**Rule:** `while` runs as long as a `bool` holds.

```
while $n > 0:
    change $n = $n - 1;
end while
```

The Precog limb can ask for `decreases`, to prove it ends.

**From:** D128.

## 4.7 `forever`

**Rule:** `forever` has no end, unless you `break`, so searching for it finds every endless loop.

```
forever:
    …
end forever
```

**From:** D128.

## 4.8 `break`

**Rule:** `break;` stops the innermost loop. A loop can be named, and `break` with that name stops it and every loop inside it.

```
for rows $row in $grid:
    for $cell in $row:
        if $cell == $target:
            break rows;
        end if
    end for
end for rows
```

**From:** D128.
