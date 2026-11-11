# 11. Errors

| Section | What | In short |
|---|---|---|
| [11.1](#111-the-layout) | The layout | One format, building or running |
| [11.2](#112-kinds) | `refused:`, `unproven:`, `check failed:`, `warning:` | The first word |
| [11.3](#113-the-place) | `file:line:col` | Written twice |
| [11.4](#114-labels) | `what:`, `why:`, `when:` | What each line says |
| [11.5](#115-fixes) | `fix:` | Only when there's exactly one |
| [11.6](#116-because) | `because:` | What led here |
| [11.7](#117-every-error) | Every error | Cascades found by applying fixes |

These come from Radium's format, as its design session described it on 2026-11-04.

## 11.1 The layout

**Rule:** building and running share one layout.

```
refused: main.cy:12:5, line 12 column 5
12 |     close(Give $f);
   |     ^^^^^^^^^^^^^^
 what: `$f` was already given away
 why:  a given value is dead afterwards (D2)
 because: main.cy:9:5, line 9 column 5
 9 |     send(Give $f);
   |     ^^^^^^^^^^^^^
     `$f` was given away here
```

There are no error codes. A `^` run sits under exactly the part in question.

**From:** D16.

## 11.2 Kinds

**Rule:**

| Kind | When |
|---|---|
| `refused:` | While building, a definite mistake |
| `unproven:` | While building, the prover could prove it neither way |
| `check failed:` | While running |
| `warning:` | Cyborg's addition |

**From:** D16.

## 11.3 The place

**Rule:** the place is written twice: `file:line:col` for tools, with the column in UTF-8 bytes, then `line N column M` for people, with the column in grapheme clusters.

**From:** D16.

## 11.4 Labels

**Rule:**

| Label | Says | When |
|---|---|---|
| `what:` | What's wrong | Always, unless `fix:` replaces it |
| `why:` | The rule behind it, citing its decision | Always |
| `when:` | The values that break it | Where known |

**From:** D16.

## 11.5 Fixes

**Rule:** a message says only what is certain. With exactly one possible fix, `fix:` replaces `what:` and shows it as code. With more than one, there are no suggestions. The `FinalShit` curse is exempt ([10.8](10-build-modes.md#108-the-curse)).

```
refused: main.cy:1:15, line 1 column 15
 1 | let $a: f64.stable = 0.1;
   |               ^^^
 fix:  `~0.1`
 why:  f64.stable can't hold 0.1 exactly (D59)
```

**From:** D16, D59.

## 11.6 `because:`

**Rule:** `because:` links show earlier steps that led to the error, in the order they happened. One appears only when the compiler knows that step led here.

**From:** D16.

## 11.7 Every error

**Rule:** building shows every error, not just the first, in the oracle and the compiler alike. A later error caused by an earlier one is shown with a `because:` pointing back at it.

After an error, checking skips only that statement and goes on with the next. A skipped `let` or `decl` still creates its variable, with the type it wrote.

To find those, the checker applies the first error's `fix:` in memory and checks again. Any later error that disappears was caused by it. The ones still there are independent, and the same is done for the next one. An error with no single fix can't be applied, so the errors after it stay independent.

```
refused: main.cy:4:25, line 4 column 25
 4 |     let mut $sum: i32 = totl(Lend $xs);
   |                         ^^^^
 fix:  `total`
 why:  a name has to be declared before it's used (the sketch)

refused: main.cy:9:5, line 9 column 5
 9 |     change $sum = $sum + $more;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^
 what: `$sum` never got a value
 why:  a variable is read only after it has one (D203)
 because: main.cy:4:25, line 4 column 25
 4 |     let mut $sum: i32 = totl(Lend $xs);
   |                         ^^^^
     with `total` here, this error goes away
```

Cascades are defined this way, not guessed, so the oracle and the compiler agree byte for byte.

**From:** D16, D207, D215.
