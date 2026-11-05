# The Cyborg standard

What Cyborg does now, in one place. The oracle, the compiler and check claude follow this. `DECISIONS.md` holds the history, with why and the owner's words. If this and a decision conflict, that's a bug in this standard, because the owner makes the decisions. (D72)

**Looking for something?** Find the word below. If it's not here, it's probably not decided: see [12](12-not-decided.md). Each file starts with a table of its sections, and each section has the same labels: **Rule**, **Refused** and **From**.

## Files

| File | What |
|---|---|
| [1. Programs](1-programs.md) | `main`, blocks, `$`, paths, narrowings, brackets, labels |
| [2. Values and types](2-values-and-types.md) | Variables, constants, numbers, strings, collections, `as` |
| [3. Operators](3-operators.md) | Every operator |
| [4. Control flow](4-control-flow.md) | `if`, the test, `match`, `for` |
| [5. Functions](5-functions.md) | Declaring, calling, contracts, methods, builtins |
| [6. Ownership](6-ownership.md) | `Give`, `Copy`, `Lend`, `%`, `must_Give` |
| [7. Structs and unions](7-structs-and-unions.md) | `struct`, `union.tagged` |
| [8. Failures](8-failures.md) | `fails`, `fail`, `?`, `??`, `otherwise`, handlers |
| [9. Limbs and shields](9-limbs-and-shields.md) | `@`, `std`, `@csup`, `shield`, Precog, the linter |
| [10. Build modes](10-build-modes.md) | The five modes, and **the one table of what's UB where** |
| [11. Errors](11-errors.md) | The message format |
| [12. Not decided](12-not-decided.md) | Anything here means `NotDesignedYet` |

## Words and symbols

| Word | Section | | Word | Section |
|---|---|---|---|---|
| `$` | [1.4](1-programs.md#14-names-and-) | | `inf` | [2.10](2-values-and-types.md#210-lists) |
| `%` | [6.8](6-ownership.md#68--unsaid-ownership) | | `Lend` | [6.4](6-ownership.md#64-lend-and-bare-parameters) |
| `*` (handlers) | [8.8](8-failures.md#88--handlers-that-jump) | | `Lend from` | [6.10](6-ownership.md#610-stored-borrows) |
| `+`, `-` | [3.2](3-operators.md#32-arithmetic----x) | | `Lend.mut` | [6.5](6-ownership.md#65-lendmut-and-lentmut) |
| `.` | [1.6](1-programs.md#16-narrowings-) | | `len`, `len.max`, `len.min` | [2.10](2-values-and-types.md#210-lists), [2.11](2-values-and-types.md#211-arrays) |
| `.%` | [5.6](5-functions.md#56-methods-) | | `Lent.mut` | [6.5](6-ownership.md#65-lendmut-and-lentmut) |
| `:` (block) | [1.3](1-programs.md#13-blocks-and-statements) | | `let` | [2.1](2-values-and-types.md#21-let) |
| `:` (label) | [1.9](1-programs.md#19-labels-) | | `let mut` | [2.2](2-values-and-types.md#22-let-mut-and-change) |
| `:` (the test) | [4.2](4-control-flow.md#42-the-test-if-n-t) | | linter | [9.11](9-limbs-and-shields.md#911-the-linter) |
| `::` | [1.5](1-programs.md#15-paths-) | | `main` | [1.2](1-programs.md#12-main) |
| `:=` | [2.3](2-values-and-types.md#23-constants-) | | `match` | [4.3](4-control-flow.md#43-match) |
| `;` | [1.3](1-programs.md#13-blocks-and-statements) | | methods | [5.6](5-functions.md#56-methods-) |
| `<`, `>` | [3.5](3-operators.md#35-comparison) | | `min`, `max` | [2.5](2-values-and-types.md#25-bounded-integers) |
| `< >` | [1.8](1-programs.md#18-brackets) | | `mod` | [3.4](3-operators.md#34-remainder-mod) |
| `=` (set) | [2.1](2-values-and-types.md#21-let), [7.2](7-structs-and-unions.md#72-building-a-struct) | | `must_Give` | [6.9](6-ownership.md#69-must_give) |
| `==`, `!=` | [3.5](3-operators.md#35-comparison) | | narrowings | [1.6](1-programs.md#16-narrowings-) |
| `?` | [8.4](8-failures.md#84--pass-it-up) | | `narrowings.failure` | [8.3](8-failures.md#83-named-sets-narrowingsfailure) |
| `??` | [8.5](8-failures.md#85--use-this-instead) | | `narrowings.handler` | [8.7](8-failures.md#87-handler-narrowings) |
| `@` (builtin) | [5.7](5-functions.md#57-builtins-) | | `not`, `and`, `or` | [3.6](3-operators.md#36-logic) |
| `@` (limb) | [9.2](9-limbs-and-shields.md#92--a-limb) | | `otherwise` | [8.6](8-failures.md#86-otherwise) |
| `[ ]`, `( )` | [1.8](1-programs.md#18-brackets) | | `plus.wrap` | [3.8](3-operators.md#38-wrapping-pluswrap) |
| `~` | [2.7](2-values-and-types.md#27-inexact-literals-) | | Precog | [9.10](9-limbs-and-shields.md#910-precog) |
| `@csup` | [9.5](9-limbs-and-shields.md#95-c-support-csup) | | `refused:` | [11.2](11-errors.md#112-kinds) |
| `@std` | [9.3](9-limbs-and-shields.md#93-stds-two-parts) | | `requires`, `ensures` | [5.4](5-functions.md#54-the-top-of-the-body), [5.5](5-functions.md#55-contracts) |
| `as` | [2.14](2-values-and-types.md#214-conversion-as) | | `return` | [5.3](5-functions.md#53-return) |
| `at_least`, `at_most` | [3.5](3-operators.md#35-comparison) | | `shield` | [9.7](9-limbs-and-shields.md#97-shields) |
| `because:` | [11.6](11-errors.md#116-because) | | `shl`, `shr` | [3.7](3-operators.md#37-bits-and-shifts) |
| `bit_and`, `bit_or`, `bit_xor`, `bit_not` | [3.7](3-operators.md#37-bits-and-shifts) | | `std::array` | [2.11](2-values-and-types.md#211-arrays) |
| `bool` | [2.15](2-values-and-types.md#215-void-and-bool) | | `std::list` | [2.10](2-values-and-types.md#210-lists) |
| build modes | [10.1](10-build-modes.md#101-the-modes) | | `std::map` | [2.12](2-values-and-types.md#212-maps) |
| `case` | [4.3](4-control-flow.md#43-match), [8.6](8-failures.md#86-otherwise) | | `str` | [2.9](2-values-and-types.md#29-strings) |
| `change` | [2.2](2-values-and-types.md#22-let-mut-and-change) | | `struct` | [7.1](7-structs-and-unions.md#71-declaring-a-struct) |
| `check failed:` | [11.2](11-errors.md#112-kinds) | | `union.tagged` | [7.5](7-structs-and-unions.md#75-tagged-unions) |
| `Copy` | [6.3](6-ownership.md#63-copy) | | `union.untagged` | [9.6](9-limbs-and-shields.md#96-cs-union) |
| curse | [10.8](10-build-modes.md#108-the-curse) | | UB | [10.4](10-build-modes.md#104-what-happens-when-something-goes-wrong) |
| `d32` | [2.8](2-values-and-types.md#28-decimal-floats) | | `unproven:` | [11.2](11-errors.md#112-kinds) |
| `div` | [3.3](3-operators.md#33-division-div) | | `void` | [2.15](2-values-and-types.md#215-void-and-bool) |
| `empty` | [2.10](2-values-and-types.md#210-lists) | | `warning:` | [11.2](11-errors.md#112-kinds) |
| `end` | [1.3](1-programs.md#13-blocks-and-statements) | | `what:`, `why:`, `when:` | [11.4](11-errors.md#114-labels) |
| `f32`, `f64` | [2.6](2-values-and-types.md#26-floats) | | `Working`, `FinalSafe`, `FinalPerf` | [10.1](10-build-modes.md#101-the-modes) |
| `fail` | [8.2](8-failures.md#82-fail) | | `FinalShit` | [10.7](10-build-modes.md#107-finalshit) |
| `fails` | [8.1](8-failures.md#81-fails) | | `FinalSmall` | [10.1](10-build-modes.md#101-the-modes) |
| `fix:` | [11.5](11-errors.md#115-fixes) | | `x` | [3.2](3-operators.md#32-arithmetic----x) |
| `fn` | [5.1](5-functions.md#51-declaring) | | | |
| `for` | [4.4](4-control-flow.md#44-for) | | | |
| `Give`, `Given` | [6.2](6-ownership.md#62-give-and-given) | | | |
| `i32`, `i64` | [2.4](2-values-and-types.md#24-integers) | | | |
| `if` | [4.1](4-control-flow.md#41-if) | | | |
