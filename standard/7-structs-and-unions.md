# 7. Structs and unions

| Section | What | In short |
|---|---|---|
| [7.1](#71-declaring-a-struct) | `struct` | Named fields |
| [7.2](#72-building-a-struct) | `reading[ ]` | `=` sets a field |
| [7.3](#73-reading-a-field) | `$r.celsius` | A plain `.` |
| [7.4](#74-linear-structs) | `struct.must_Give` | Handed off exactly once |
| [7.5](#75-tagged-unions) | `union.tagged` | One of several cases |
| [7.6](#76-using-a-union) | `status.failed` | Cases are narrowings |
| [7.7](#77-no-bare-union) | No bare `union` | Just say it |

## 7.1 Declaring a struct

**Rule:** fields have no `$`.

```
struct reading:
    celsius: i32;
    time: i64.std::units::time.seconds;
end struct reading
```

**From:** D57.

## 7.2 Building a struct

**Rule:** building one uses `[ ]`, because it holds values and runs nothing. `=` sets a field.

```
let $r: reading = reading[celsius = 21, time = 3600 std::units::time.seconds];
```

**From:** D23, D57.

## 7.3 Reading a field

**Rule:** a field read is a plain `.`, and no code runs.

```
std::io::fmt_print("{}{newln}", $r.celsius);
```

A field can be changed if the whole variable is `let mut`, or the parameter `Lent.mut`:

```
change $r.celsius = 22;
```

**From:** D8, D57, D125.

## 7.4 Linear structs

**Rule:** `struct.must_Give` declares a linear type ([6.9](6-ownership.md#69-must_give)).

**From:** D2, D57.

## 7.5 Tagged unions

**Rule:** a tagged union holds one of its cases, and carries a hidden tag saying which. A case's values go in `[ ]`.

```
union.tagged status:
    ok
    failed[code: i32]
end union.tagged status
```

**From:** D33.

## 7.6 Using a union

**Rule:** cases are narrowings, such as `status.failed`. You reach a case with `match` ([4.3](4-control-flow.md#43-match)) or the test ([4.2](4-control-flow.md#42-the-test-if-n-t)).

**From:** D9, D33.

## 7.7 No bare union

**Rule:** there's no bare `union`, because "union" means opposite things to a C programmer and a Rust one. C's untagged union is in [9.6](9-limbs-and-shields.md#96-cs-union).

**Refused:** `union status:`.

**From:** D33.
