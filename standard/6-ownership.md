# 6. Ownership

| Section | What | In short |
|---|---|---|
| [6.1](#61-the-words) | All the words | One table |
| [6.2](#62-give-and-given) | `Give`, `Given` | Ownership moves |
| [6.3](#63-copy) | `Copy` | A separate copy |
| [6.4](#64-lend-and-bare-parameters) | `Lend`, bare | Read-only borrow |
| [6.5](#65-lendmut-and-lentmut) | `Lend.mut`, `Lent.mut` | Borrow that may change it |
| [6.6](#66-plain-numbers-and-bool) | Plain numbers, `bool` | Always copied, no word |
| [6.7](#67-no-implicit-ownership) | No implicit ownership | Every hand-off shows |
| [6.8](#68--unsaid-ownership) | `%` | Unsaid ownership on a method |
| [6.9](#69-must_give) | `must_Give` | Handed off exactly once |
| [6.10](#610-stored-borrows) | `Lend from` | A borrow as a type |
| [6.11](#611-no-word-needed) | No word needed | Reading in place, fresh values |

## 6.1 The words

**Rule:** the call and the parameter both say it, and they must match.

| At the call | On the parameter | Meaning |
|---|---|---|
| `Give $x` | `Given` | Ownership moves, and `$x` can't be used afterwards |
| `Copy $x` | `Given` | The function owns a separate copy, and the caller's is untouched |
| `Lend $x` | (bare) | Temporary read-only access, and the caller keeps it |
| `Lend.mut $x` | `Lent.mut` | Temporary access that may change it, and nothing else uses it meanwhile |

These are Radium's words, unchanged.

**From:** D2.

## 6.2 `Give` and `Given`

**Rule:** `Give` moves ownership, and the value can't be used afterwards, on any path.

```
fn close [$f: Given open_file] -> void:
    …
end fn close

close(Give $f);
```

**Refused:** using `$f` after `Give $f`.

**From:** D2.

## 6.3 `Copy`

**Rule:** `Copy` hands over a separate copy, to a `Given` parameter. There is no parameter word for `Copy`.

**Refused:** `Copy` on an integer or `bool` ([6.6](#66-plain-numbers-and-bool)).

**From:** D2.

## 6.4 `Lend` and bare parameters

**Rule:** a bare parameter is lent, read-only, and the call still writes `Lend`.

```
fn total [$xs: std::list<i32>[len.max: inf items]] -> i32:
    …
end fn total

let $t: i32 = total(Lend $xs);
```

**Refused:** writing `Lent` alone on a parameter, because it's the default.

**From:** D2.

## 6.5 `Lend.mut` and `Lent.mut`

**Rule:** the function may change the value, the caller keeps it, and nothing else uses it meanwhile.

```
fn add_row [$t: Lent.mut transaction] -> void:
    …
end fn add_row

add_row(Lend.mut $t);
```

**From:** D2.

## 6.6 Plain numbers and `bool`

**Rule:** they're always copied, and take no word. That covers `i32`, `f64`, bounded integers, numbers with units and `d32`.

**Refused:** any ownership word on them.

**From:** D2, D123.

## 6.7 No implicit ownership

**Rule:** every hand-off shows on the page, including one to a method's receiver, into a name, a `return` or a field.

`change` doesn't stand in for an ownership word. A value handed into a name, a `return` or a field takes `Give` or `Copy`. `Lend` needs a type that's a stored borrow ([6.10](#610-stored-borrows)).

```
let $k: str = Copy $s;
return Give $t;
name_tag[text = Give $s]
```

**Refused:** `change $xs = $xs.append(val: 4)`, and a whole struct or list passed or stored with no word.

**From:** D2, D3.

## 6.8 `%`: unsaid ownership

**Rule:** a `%` on a method's name says the receiver's ownership is unsaid here, and found in the signature. The compiler still checks it. It hides only `Lend` and `Lend.mut`, so after a `%` the value is always still alive. A search for `%` finds every unsaid hand-off.

```
$xs.std::%append(val: 4);
```

**Refused:** `total(%$xs)`, because `%` goes only on a method call. Hiding `Give`: write `close(Give $f)`.

**From:** D4, D5, D115.

## 6.9 `must_Give`

**Rule:** a `must_Give` type is linear: it's handed off with `Give` exactly once. Lending doesn't count.

```
struct.must_Give open_file:
    …
end struct.must_Give open_file
```

**Refused:** forgetting it, or giving it twice, as a compile error.

**From:** D2, D57.

## 6.10 Stored borrows

**Rule:** a stored borrow is a type.

```
Lend from $r reading
Lend from <$a, $b> reading
Lend from program reading
Lend.mut from $r reading
```

**From:** D2.

## 6.11 No word needed

**Rule:** reading in place, and a fresh value that has no holder yet, take no word.

```
$xs.len
for $x in $xs:
reading[celsius = 21, time = 3600 std::time.seconds]
```

**From:** D2.
