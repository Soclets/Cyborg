# 8. Failures

| Section | What | In short |
|---|---|---|
| [8.1](#81-fails) | `fails` | Lists what can fail, first in the body |
| [8.2](#82-fail) | `fail` | Raises one |
| [8.3](#83-named-sets-narrowingsfailure) | `narrowings.failure` | A reusable set |
| [8.4](#84--pass-it-up) | `?` | Pass it to the caller |
| [8.5](#85--use-this-instead) | `??` | Use this value instead |
| [8.6](#86-otherwise) | `otherwise:` | Handle it in a block |
| [8.7](#87-handler-narrowings) | `narrowings.handler` | Reusable arms |
| [8.8](#88--handlers-that-jump) | `.*` | This handler can jump |

## 8.1 `fails`

**Rule:** a function that can fail lists its failures in its body, after `requires` and `ensures` ([5.4](5-functions.md#54-the-top-of-the-body)). The list can mix named sets and one-off failures, and a failure can carry data in `[ ]`.

```
fn load [$path: str] -> config:
    fails [file_failed, bad_syntax];
    …
end fn load

fn half [$n: i32] -> i32:
    fails [odd, negative[value: i32]];
    …
end fn half
```

**Refused:** `-> i32 or fail[…]`, the older spelling.

**From:** D47, D52.

## 8.2 `fail`

**Rule:** `fail` raises a failure, and leaves the function.

```
fail odd;
fail negative[value = $n];
```

**From:** D52.

## 8.3 Named sets: `narrowings.failure`

**Rule:** a named set of failures is a collection.

```
narrowings.failure file_failed:
    file_failed.not_found;
    file_failed.denied;
    file_failed.disk_full[free: i32];
end narrowings.failure file_failed
```

Two sets sharing a failure's name need the full name in a list: `file_failed.denied`.

**From:** D47.

## 8.4 `?`: pass it up

**Rule:** if this failed, hand the failure to the caller. There's no word form.

```
let $f: file = open(Lend $path)?;
```

**Refused:** a `?` passing up a failure the function doesn't list.

**From:** D44, D45, D47.

## 8.5 `??`: use this instead

**Rule:** if this failed, use this instead. There's no word form.

```
let $port: i32 = read_port() ?? 8080;
```

**From:** D44, D45.

## 8.6 `otherwise:`

**Rule:** `otherwise:` handles a failure in a block. Its arms must cover every failure still possible at that call.

```
let $w: str = first_word(Lend $text) otherwise:
    case too_long:
        "…";
    case empty:
        "(none)";
end otherwise
```

An arm ending in a value, and how an arm reads a failure's data, are shown from Radium and aren't decided ([12.6](12-not-decided.md#126-failures)).

**From:** D45, D46.

## 8.7 Handler narrowings

**Rule:** a handler narrowing is a named set of `otherwise` arms, used at a call as if written there. The spelling `narrowings.handler` is Radium's.

```
narrowings.handler word_failed:
    word_failed.use_placeholder:
        case too_long:
            "…";
        case empty:
            "(none)";
    end word_failed.use_placeholder
end narrowings.handler word_failed

let $w: str = first_word(Lend $text) otherwise word_failed.use_placeholder;
```

**From:** D45, D46.

## 8.8 `*`: handlers that jump

**Rule:** a handler that can `return`, `break` or `continue` is marked `*` where it's used, and not where it's declared.

```
let $w: str = first_word(Lend $text) otherwise word_failed.*bail;
```

**Refused:** leaving `*` off a handler that can jump, and putting it on one that can't.

**From:** D46.
