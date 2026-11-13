# "Merging with the Machine" on the Uno R4 WiFi's LED matrix

Scrolls Cyborg's tagline across the board's 12x8 LED matrix. First ran on 2026-11-13, built by `OKAY\n` with
`--FinalPerf --target=arduino_uno_r4_wifi`, and flashed as in `examples/uno_r4_blink`.

The matrix is 96 LEDs on 11 chip pins, charlieplexed: each LED sits between two pins and lights when one is driven high
and the other low, so one is on at a time and they're scanned fast enough to look like a picture. Which pins light which
LED comes from Arduino's own `Arduino_LED_Matrix` library, and each pin is set through its own PFS register.

`matrix.cy` is made from `matrix.cy.in` by `make_tables.py.txt`, a scratch Python script kept as text since rule 1 keeps
committed tools in Rust. It draws a small font and writes the message's columns and the pin tables as constants.

## What it ran into

- Converting between integer types with `as`, like `i32` to `u8`, isn't decided yet (12.3), so everything is `i32`,
  except pointer addresses, which `.at(address:)` takes as `u64`.
- A constant array with long runs of zeros made the compiler call `memset`, which its bare-metal runtime doesn't have,
  so every column has bit 8 set, which the display never reads.
- Even then, `FinalSafe` still needed `memset` when the program used both kinds of table, so this is built with
  `FinalPerf`. Its indexes are in range by construction.
