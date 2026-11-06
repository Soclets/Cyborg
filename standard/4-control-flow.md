# 4. Control flow

| Section | What | In short |
|---|---|---|
| [4.1](#41-if) | `if` | Runs a block when true |
| [4.2](#42-the-test-if-n-t) | `if $n: T:` | Tests a type, and trusts it inside |
| [4.3](#43-match) | `match` | Must cover every case |
| [4.4](#44-for) | `for` | Goes through a collection |

`while`, `loop`, `break`, `continue`, `else` and ranges aren't decided ([12.4](12-not-decided.md#124-control-flow)).

## 4.1 `if`

**Rule:** `if` runs its block when the condition is true.

```
if $t at_least 10:
    std::println("big total: ", $t as str);
end if
```

**From:** the sketch.

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

**Refused:** a `match` that misses a case.

**From:** D9, D33.

## 4.4 `for`

**Rule:** `for` goes through a collection, reading it in place, so it needs no ownership word.

```
for $x in $xs:
    change $sum = $sum + $x;
end for
```

**From:** D2, the sketch.
