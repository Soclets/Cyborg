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

These are places where decisions leave a real gap or pull against each other.

| Problem | Decisions |
|---|---|
| **Expensive contracts in `FinalSafe`.** D14 says contracts are checked, and `FinalSafe` keeps only cheap checks. Only a shield's exit is settled. | D14, D37, D39 |
| **What `FinalShit` drops.** Does it also drop `%`, a missing `~`, or mismatched `Give` and `Lend`? D40's list says "such as". | D5, D40, D59 |
| **Labels and parameters.** The sketch calls `append(val: 4)` with a label and `total(Lend $xs)` without one. Nothing says when a label is required, or how it relates to a parameter's name. | D13, D23 |

## 12.2 Types

- The full list of integer and float types.
- Units (D99 to D101): multiplying and converting, how a unit is defined, and the full list of units `std` has.
- Raw pointers' spelling, though D38 puts them in shields.
- Function types: `-> i32` no longer shows failures, so a function's type needs some way to include them.
- How a type's writer defines label narrowings like `len.max`.
- An uninitialized array, like Zig's `= undefined`. Claude suggested only inside a shield, if at all.
- Indexing, and what a bad index does.
- `str` narrowings other than `str.utf16`, and how string literals work.
- The full print (D98): how `std` spells the stream, the buffer and flushing.
- `true` and `false`.
- The test on a value already known to be that type, like `if $n: bool:` on a `bool`. Claude suggested refusing it.

## 12.3 Operators and conversion

- Operator precedence.
- Whether `-` and `x` follow [10.4](10-build-modes.md#104-what-happens-when-something-goes-wrong)'s overflow row, which D42 states for `+`.
- The real spelling for wrapping, and the wrapping forms of `-` and `x`.
- The full list of `div` narrowings, and what `div.exact` does in each mode.
- Shifting by too much.
- Unary minus.
- What can convert to what with `as`.

## 12.4 Control flow

- `else`.
- `while`, `loop`, `break` and `continue`.
- Ranges. Claude proposed `range.inclusive` and `range.exclusive`, with no bare `range`.
- Where the test in [4.2](4-control-flow.md#42-the-test-if-n-t) is allowed besides `if`.

## 12.5 Functions

- How a method is declared, and how its receiver appears in the signature.
- Generics (D85, D87): the spelling, including `?T` and declaring a bound, how a bound is defined, whether you can declare one for a type from someone else's kit, and how specialization and value parameters are written.
- Whether a `fn.pure` may allocate or fail.
- The inline compile-time language (D88 to D91): its syntax.
- Whether every `void` function says `return void;`.
- When a call needs labels ([12.1](#121-problems-found-writing-this)).
- Comments. Examples in the decisions use `//`, but it was never chosen.
- Naming rules, like capitals.

## 12.6 Failures

- How an arm reads a failure's data.
- What an arm's last line means, such as `"…";` used as the value.
- Whether `??` can take a call, not just a value.
- `?` in a function that lists no failures, and in `main`.
- The list of a handler narrowing's arguments, if any.

## 12.7 Build modes

- Whether `FinalSafe` keeps expensive contracts ([12.1](#121-problems-found-writing-this)).
- Whether `FinalSmall` treats contracts as facts.
- What `FinalShit` drops beyond D40's list ([12.1](#121-problems-found-writing-this)).
- How you choose optimization in `Working`.
- Whether an unhandled failure still crashes in `FinalShit`, which assumes nothing bad can happen.
- What the failure is called when a list is full, or at `len.min` when removing.

## 12.8 Ownership

These are inherited from Radium with D2:

- Lending through a stored `Lend.mut`.
- How a `must_Give` duty ends in the function that receives it.
- A `must_Give` value in a field or returned.
- A parameter whose type is a stored borrow.
- A `union.tagged` case holding a borrow.
- Going through items that are structs, unions or arrays.

## 12.9 Limbs and projects

- Modules, and how files see each other.
- The manifest's format and name. D30, D35 and D81 rely on it.
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

- The file extension `.cy`.
- The command `cyborg build`.
- `key` and `val` in `std::map`.
- `plus.wrap`.
- `result`.
- The repository's name, and the license.
