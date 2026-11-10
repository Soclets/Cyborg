# 12. Not decided

**Anything here means `NotDesignedYet`.** Nothing here gets guessed. It goes to the owner through design claude.

| Section | What |
|---|---|
| [12.1](#121-problems-found-writing-this) | **Problems found writing this** |
| [12.2](#122-types) | Types |
| [12.3](#123-operators-and-conversion) | Operators and conversion |
| [12.4](#124-control-flow) | Control flow |
| [12.5](#125-functions) | Functions |
| [12.6](#126-failures) | Failures |
| [12.7](#127-build-modes) | Build modes |
| [12.8](#128-ownership) | Ownership |
| [12.9](#129-limbs-and-projects) | Limbs and projects |
| [12.10](#1210-errors) | Errors |
| [12.11](#1211-placeholders) | Placeholders |

## 12.1 Problems found writing this

These are places where decisions leave a real gap or pull against each other. None are open right now.

### Queue from oracle claude

Waiting on the owner, most needed first. Each is `NotDesignedYet` unless it says otherwise.

- **Precedence fixes.** What `??`, `?` and `otherwise` catch on more than one call, like `(f() + g()) ?? 0`. `as` and `plus.wrap` have no level; the oracle treats `as` as tight, and `plus.wrap` as level with `+`.
- **`*` on handlers.** Whether `fail` and `?` count as jumps. Today they don't.
- **Fields.** Whether `for` over `$b.items` uses all of `$b`; `change $s.code` inside its test, and whether trust survives. The oracle allows `change` through a stored `Lend.mut`, which is its reading.
- **Trust in `else:`.** With two cases, whether `else:` knows `$s` is the other one, and whether trust carries on after a block that always leaves. Both are refused today.
- **What `$i` is** in `range[from: 1, to: $xs.len]`. It's `u64.items` today, which is a reading.
- **`as`, readings.** Crossing only goes between a `type.new` and exactly what it's made from, or a unit on or off the same width. A `type.new` over a bound can go out but not in, since crossing can't fail. A literal lands on numbers, strings, `[ ]`, `empty` and `true`, but `wrapped[celsius = 1]` isn't one.
- **Unit labels, readings.** The unit in a label has to match the parameter's type. A plain variable takes the label's unit too. `pos.items:` is `NotDesignedYet`. `range[from: $n, to: $xs.len]` mixing a plain variable and a length is `NotDesignedYet`.
- **Ranges.** How to name `while` and `forever`, like `forever rows:`. A `step_by` that jumps past the end runs 1, 3, 5 today, and `range[from: 2, before: 2]` runs zero times, which are readings. Ranges of floats, units and lengths. A range anywhere but after `in`. A nested loop reusing an outer one's name.
- **Handlers and loops.** Today a handler only needs `*` if its `break` leaves the arm, and `while true:` still needs a `return` after it, because only `forever` promises not to finish. Both are readings.
- **`finish`.** Whether a value can be used after `finish`. `finish` on no duty, a lent value and a fresh value is refused today, and it's allowed on a local you made, which are readings. Its two other refusals can't be reached yet: one needs modules, and one needs a `must_Give` in a field.
- **Methods.** On a type you didn't declare, like `fn i32.%double`, or a list. Calling one by its full name, `box.%read(Lend $b)`. `$b.%%add(by: $b.n)`, reading the receiver in an argument while it's lent with `Lend.mut`, which people will write a lot, and Rust calls two-phase borrows. `Lent.mut` on a fresh value.
- **`??` and `otherwise`.** `read() ?? -$x` is refused and asks for `( )`, and `read() ?? 1 as str` is refused too, so should a `??` count as a single value? `otherwise` has no level in 3.9, so `$a + read() otherwise:` isn't decided.
- **Positions.** `$a[pos: 0]` and `$a[pos: 9]` on an array of 3 are `check failed:` today, but D20 refuses a literal that can't fit a bound, so maybe they should be refused too. Two items lent at once, like a swap, and changing one item while a `for` goes through another. `Give $a[pos: 1]`, and taking something out of a list. `change $xs[pos: 1] = grow(Lend.mut $xs);`. Handing something off inside a position.
- **`type.new`.** Whether maths and `==` work between two of the same one.
- **Type names, readings.** `type.alias int.signed.32_bit = i32;` works, with dots in the name. An alias works everywhere its target does, including cases. A `type.new` over a number takes no word, over a struct needs one, and over a `must_Give` carries the duty.
- **Type names, smaller.** A method on an alias or a family's type. A narrowing or bounds on a name. `type.…` inside a function. A `type.new` holding itself through a list. An alias named like a struct. Declaring a name twice.
- **`fam.fn`.** D135 includes `fn` collections, but no spelling's shown.
- **`Copy`'s depth.** Whether copying a struct holding a list copies the list too. Other languages disagree on this.
- **Reading a case (D139).** Whether `case status.failed: … $s.code` with no name is still allowed. Whether `$f` is a read-only view, as it is today, or a copy. `if $s: status.failed $f:`. `instead` inside a loop inside its arm.
- **Arms, readings.** An arm that can run off its end where a value's needed is refused. A bare value as an arm's last line is refused, with `fix: instead …`. `instead` on a call giving nothing back is refused. `*` on a handler with only `instead` is refused. What an arm gives away is only maybe gone afterwards. A failure's `$f` reads data only, and non-numbers need a word: `instead Copy $x.label;`.
- **Lengths and units (the gap felt most).** `$a.len - 1 items` going below zero, since `-`'s overflow isn't stated, and `$a.len x 2`. An `inf` list outgrowing its `len.type`, like the 256th `append` with `len.type: u8`.
- **`len.type`, readings.** It takes any integer in the list, including `i32`, and goes only on the collection. Lists differing only in `len.type` are different types.
- **Steps and quoted names, readings.** A written `step_by: 0` is refused even when the other numbers are only known while running. A step of 0 is a `check failed:` with no name, can't be handled, and wins over zero rounds. A float or unit step is `NotDesignedYet`. After `as "secs"`, the type is `i32.secs` as well as the value `3600 secs`. `(Give $r) as wrapped` is refused, and `Give $h.inner as wrapped` is `NotDesignedYet`.
- **How close `f64.fast` has to be.** Its tests check that the answer is close (D163), and how close isn't stated.
- **Floats, what's stuck.** `$x as f64.ieee`, which can't fail. A bound on `f64.ieee`. `$reading as f64[min: 0.0, max: 2.0]`. How `f64.fast` mixes with `f64`. A literal too large for `f64.ieee`.
- **Floats, readings.** NaN or infinity is a `check failed:` nothing can handle, and refused in a `const`. A failed `$reading as f64` is a `check failed:` too. Float bounds behave like integers, and D59 applies, so `f64[min: 0.1, …]` needs `~0.1`. `f32.ieee` and `f32.fast` work too. `f64.fast` can't be NaN or infinity either.
- **`use`, readings.** D151 refuses every reserved word, including `"i32"`, plus `"2nd"` and `"$say"`. A limb still needs its `import`, and a short name still needs a shield where the limb would. `use @csup::union.untagged as untagged;` gives `untagged number:`, closed with `end untagged number`.
- **Dead variables.** Whether `eat(Give $s); change $s = "b";` is allowed. It's refused today.
- **Names.** Declaring a name already in scope, or hiding an outer one.
- **`ensures` on a `fail` path.** Checked only on `return` today, which is a guess.
- **Evaluation order.** Left to right today, which is a guess.
- **Dropped values.** `read();` and `read() ?? void;` on a call that returns something.
- **Handling what can't fail.** `?`, `??` and `otherwise` on a call that can't fail.
- **A failed `as`.** What the failure is called.
- **`Given` parameters.** They can't change without `let mut $ys: … = Give $xs;`. This follows the rules, and is only flagged.
- **Smaller ones.** A bound with only `min` or only `max`; a bound on a number with a unit; `my::` on a name that doesn't clash; building fields out of order; two `case`s for one failure; `case file_failed:` covering a set; `match` on a non-union; a variable typed `status.failed`; `==` on `str` or structs; `<` on `bool`; `shr` on a negative; `shl` into the sign bit; the lowest `i32` divided by -1; `all` with something other than a plain number; `std::io::fmt_print()` with no text; a struct holding itself.
- **The oracle's own limits**, so they don't get pinned by accident: 2000 levels of nesting, 10,000 call depth, 10 million compile-time steps, 16 million items in `all`, exit code 0 or 1, and stopping at the first error. Running out of stack may want a row in [10.4](10-build-modes.md#104-what-happens-when-something-goes-wrong).
- **The `NotDesignedYet:` layout**, as a first word in the D16 format.

## 12.2 Types

- The full list of integer and float types.
- Units (D99 to D111): the spelling of `together`, offsets, moments, how a conversion is written, what a lossy conversion does, and the full list of units `std` has.
- On floats: whether `div.trunc` and the other integer narrowings work, and what `mod` does.
- Negative exponents, as in `exponent ~1.5e-3`, and exponents on integers.
- Which groups `std` has (D196), besides `io` and `math`.
- A file's rules (D201): how a rule is spelled, and the list of rules.
- Whether lint rules are proven with Verus under rule 13 (D202).
- `pub` (D200): whether a closer repeats it, and finer levels like Rust's `pub(crate)`.
- Importing one thing (D198): alongside `import std;`, and with `@` or `pkg`.
- `:=` (D216): the spelling of Precog's rule, list items and struct fields in Precog's mode, and `:=` on a borrow.
- Checking past an error (D215): an error in a block's opening line, like `if` or `for`, or in a function's header.
- Spacing (D213): indentation, line length, and the space before an opening `[`, which `fn total [$xs: …]` has and `reading[celsius = 21]` doesn't.
- Named holes (D210): shapes on one, a field like `{$r.celsius}`, and mixing them with `{}` or numbered holes.
- Print (D194): how the fill character is written, a per-value `form: stored`, formatting to a `str` without printing, and what the output is on bare metal. `form: science` is how a float prints with an exponent.
- How to check a float's sign.
- Raw pointers' spelling, though D38 puts them in shields.
- Function types: `-> i32` no longer shows failures, so a function's type needs some way to include them.
- How a type's writer defines label narrowings like `len.max`.
- An uninitialized array, like Zig's `= undefined`. Claude suggested only inside a shield, if at all.
- Whether a bad position can be handled, rather than trapping.
- `str` narrowings other than `str.utf16`, and how string literals work.
- The full print (D98): how `std::io::fmt_print` takes a stream, a buffer and flushing.
- The test on a value already known to be that type, like `if $n: bool:` on a `bool`. Claude suggested refusing it.

## 12.3 Operators and conversion

- Whether `-` and `x` follow [10.4](10-build-modes.md#104-what-happens-when-something-goes-wrong)'s overflow row, which D42 states for `+`.
- The real spelling for wrapping, and the wrapping forms of `-` and `x`.
- The full list of `div` narrowings, and what `div.exact` does in each mode.
- Shifting by too much.
- What can convert to what with `as`.
- How a unit prints with `as str.with_unit`.

## 12.4 Control flow

- Where the test in [4.2](4-control-flow.md#42-the-test-if-n-t) is allowed besides `if`.

## 12.5 Functions

- Generics (D85, D87): the spelling, including `?T` and declaring a bound, how a bound is defined, whether you can declare one for a type from someone else's kit, and how specialization and value parameters are written.
- Whether a `fn.pure` may allocate or fail.
- How raw pointers, volatile memory, exact layouts and allocators are spelled (D226).
- How the code inside an `inline` block is written (D227).
- How a C function is declared and called through `@csup` (D225), and whether D220 covers a limb function's narrowing.
- Compile-time features (D218): the syntax of a `match` on types, variadic generics, and the loop over a type's fields.
- Whether every `void` function says `return void;`.
- Naming rules, like capitals.

## 12.6 Failures

- Whether `??` can take a call, not just a value.
- `?` in a function that lists no failures, and in `main`.
- The list of a handler narrowing's arguments, if any.

## 12.7 Build modes

- Whether `FinalSmall` treats contracts as facts.
- How you choose optimization in `Working`.
- What the failure is called when a list is full, or at `len.min` when removing.

## 12.8 Ownership

These are inherited from Radium with D2:

- Whether a C function's `Given` parameter ends a `must_Give` duty.

## 12.9 Limbs and projects

- Modules, and how files see each other.
- The manifest (D228): how a version number is written and what it means, what features are, the lockfile, versions as ranges, and per-target settings.
- How the manifest says which compiler a package's limb half works with.
- Whether the mark goes only at the path's start.
- Whether other limbs have a kit part like `std`.
- How a third-party kit or limb is spelled.
- Whether `@std` needs listing in a project.
- `d32`'s limb. It could be a money limb.
- The keyword `shield`.
- How the walls work for code that isn't proven, beyond [9.9](9-limbs-and-shields.md#99-a-shields-walls).
- The list of builtins.

## 12.10 Errors

- Colour.
- How many errors per run, and their order.
- Any machine-readable form.

## 12.11 Placeholders

These have a working name, but aren't chosen:

- `key` and `val` in `std::map`.
- `plus.wrap`.
- `held` on a stored borrow.
- `exponent` in front of a literal.
- `result`.
- The repository's name, and the license.
