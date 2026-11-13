# 8. Failures

| Section | What | In short |
|---|---|---|
| [8.1](#81-fails) | `fails` | Lists what can fail, in the header |
| [8.2](#82-fail) | `fail` | Raises one |
| [8.3](#83-named-sets-famfailure) | `fam.failure` | A reusable set |
| [8.4](#84--pass-it-up) | `?` | Pass it to the caller |
| [8.5](#85--use-this-instead) | `??` | Use this value instead |
| [8.6](#86-otherwise) | `otherwise:`, `instead` | Handle it in a block |
| [8.7](#87-handler-narrowings) | `fam.handler` | Reusable arms |
| [8.8](#88--handlers-that-jump) | `.*` | This handler can jump |
| [8.9](#89-crash-by-default) | No handler | A failure crashes, unless you opt in |

## 8.1 `fails`

**Rule:** a function that can fail lists its failures in its header, after the return type. The list can mix named sets and one-off failures, and a failure can carry data in `[ ]`.

```
fn load [$path: str] -> config fails [file_failed, bad_syntax]:
    …
end fn load

fn half [$n: i32] -> i32 fails [odd, negative[got: i32]]:
    …
end fn half
```

**Refused:** `returns i32`, with `fix: -> i32`, `-> i32 or fail[…]`, an older spelling, and `fails` in the body, with the fix moving it into the header.

**From:** D47, D52, D209.

## 8.2 `fail`

**Rule:** `fail` raises a failure, and leaves the function.

```
fail odd;
fail negative[got = $n];
```

**From:** D52.

## 8.3 Named sets: `fam.failure`

**Rule:** a named set of failures is a collection.

```
fam.failure file_failed:
    file_failed.not_found;
    file_failed.denied;
    file_failed.disk_full[free: i32];
end fam.failure file_failed
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
        instead "…";
    case no_words:
        instead "(none)";
end otherwise
```

`instead` gives the value used in place of the failed call. An arm can also `return`, `break` or `fail`. Naming the failure in its `case` lets the arm read its data:

```
let $h: i32 = half($n) otherwise:
    case negative $f:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $f.got);
        instead 0;
end otherwise
```

**From:** D45, D46, D139.

## 8.7 Handler narrowings

**Rule:** a handler narrowing is a named set of `otherwise` arms, used at a call as if written there. The spelling `fam.handler` is Radium's.

```
fam.handler word_failed:
    word_failed.use_placeholder:
        case too_long:
            instead "…";
        case no_words:
            instead "(none)";
    end word_failed.use_placeholder
end fam.handler word_failed

let $w: str = first_word(Lend $text) otherwise word_failed.use_placeholder;
```

**From:** D45, D46.

## 8.8 `*`: handlers that jump

**Rule:** a handler that can `return` or `break` is marked `*` where it's used, and not where it's declared.

```
let $w: str = first_word(Lend $text) otherwise word_failed.*bail;
```

**Refused:** leaving `*` off a handler that can jump, and putting it on one that can't.

**From:** D46, D128.

## 8.9 Crash by default

**Rule:** a call that fails with nothing handling it stops the program loudly. You opt in to handling it. This covers every failure, including printing, a full list, running out of memory and a failed `as`.

```
// crashes if it fails
std::io::fmt_print(Lend.mut $out, "hi{newln}");

// hands it to the caller
std::io::fmt_print(Lend.mut $out, "hi{newln}")?;

// ignores it, on purpose
std::io::fmt_print(Lend.mut $out, "hi{newln}") ?? void;

// handles it
std::io::fmt_print(Lend.mut $out, "hi{newln}") otherwise:
    case print_failed:
        return void;
end otherwise
```

Before the crash message, every buffered writer still holding output is flushed, so the last thing printed before a crash usually shows. A writer that's locked or in the middle of an operation is skipped, since flushing it could deadlock or send out half-written bytes, and each flush has a time limit, so a full pipe can't hang the crash. The crash message names each writer it skipped or that timed out. How long the limit is isn't decided ([12.6](12-not-decided.md#126-failures)). A search for `?? void` finds every ignored failure. The crash stays in every mode except `FinalShit`, where it's UB ([10.7](10-build-modes.md#107-finalshit)). The linter can be stricter, such as requiring handling ([9.11](9-limbs-and-shields.md#911-the-linter)). `print_failed` is a placeholder.

**From:** D95, D96, D107, D274, D408.
