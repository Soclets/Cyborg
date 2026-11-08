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
| [6.9](#69-must_give) | `must_Give`, `finish` | Handed off exactly once |
| [6.10](#610-stored-borrows) | `Lend from` | A borrow as a type, returned or in a struct |
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

Giving it to a function passes the duty along. `finish` ends it, and only the kit or file that declared the type can write that.

```
fn close [$f: Given open_file] -> void:
    …
    finish $f;
    return void;
end fn close
```

**Refused:** forgetting it, giving it twice, `finish` outside the kit or file that declared the type, and `finish` while a `must_Give` field inside hasn't been given away.

**From:** D2, D57, D129.

## 6.10 Stored borrows

**Rule:** a stored borrow is a type.

```
Lend from $r reading
Lend from <$a, $b> reading
Lend from program reading
Lend.mut from $r reading
```

A stored borrow ends at its last use. `.held` makes it last until the end of its block instead, even if it's never used again:

```
let mut $r: reading = reading[celsius = 21, time = 3600 std::time.seconds];
let $view: Lend from $r reading = Lend $r;
std::println($view.celsius as str);
change $r.celsius = 22;

let $lock: Lend.held from $r reading = Lend $r;
std::println($lock.celsius as str);
```

The `change` is fine, because `$view` isn't used after the print. After `$lock`, nothing can change `$r` until the block ends. `Lend.mut.held` works the same way, and `held` is a placeholder name.

A function can return a borrow by naming where it comes from. That's Cyborg's lifetime: `from $a` means it lives as long as `$a`.

```
fn warmer [$a: reading, $b: reading] -> Lend from <$a, $b> reading:
    if $a.celsius > $b.celsius:
        return Lend $a;
    end if
    return Lend $b;
end fn warmer

let $w: Lend from <$x, $y> reading = warmer(Lend $x, Lend $y);
```

- It comes only from a parameter, a named source or `program`.
- The body is checked against the claim.
- `<$a, $b>` lasts only while both do.
- At the call, the result's sources become the arguments, and each of those has to be a named value.

**Refused:** borrowing a local or a `Given` parameter, returning `Lend $b` from `-> Lend from $a`, a fresh argument the result borrows from, and a borrow of a plain number ([6.6](#66-plain-numbers-and-bool)).

A struct can hold a borrow. It names its sources in backticks, and a type fills them in, in the declared order:

```
struct index <`text`, `table`: lasts_at_least `text`>:
    words: Lend from `text` reading;
    lookup: Lend from `table` reading;
end struct index

fn build [$article: reading, $list: reading] -> index<`text`: $article, `table`: $list>:
    requires $list lasts_at_least $article;
    return index[words = Lend $article, lookup = Lend $list];
end fn build
```

- Each source is used by a field, and has its own name.
- `program` is a source that lasts until the program ends.
- A function states which parameter lasts longer with `requires … lasts_at_least …;`, and every call meets it.
- A function names a borrow inside a parameter with its own sources: ``fn shown <`t`> [$v: view<`text`: `t`>]``.

**Refused:** leaving a source out, a source no field uses, and the same `lasts_at_least` fact written twice.

Lending an item locks the whole list, and its type names the list:

```
fn first [$xs: std::list<reading>[len.min: 1 items]] -> Lend from $xs reading:
    return Lend $xs[pos: 1];
end fn first
```

Lending a field locks only that field, though its type still names the whole value. Different fields are separate:

```
adjust(Lend.mut $r.celsius, Lend.mut $r.time);
```

**Refused:** `Give $r` or `Lend.mut $r` while one of its fields is lent.

A stored borrow can be lent on, and the new borrow names the original value:

```
let $m: Lend.mut from $r reading = Lend.mut $r;
warm_up(Lend.mut $m);
change $m.celsius = 30;
```

While `Lend.mut $m` is held, `$m` can't be used. While `Lend $m` is held, `$m` can be read but not changed. `Give $m` is refused.

**From:** D2, D169, D170, D171, D172, D173, D174.

## 6.11 No word needed

**Rule:** reading in place, and a fresh value that has no holder yet, take no word.

```
$xs.len
for $x in $xs:
reading[celsius = 21, time = 3600 std::time.seconds]
```

**From:** D2.
