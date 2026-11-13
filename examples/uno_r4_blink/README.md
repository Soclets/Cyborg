# Blinking the Uno R4 WiFi

Cyborg's first program on real hardware, D252's first milestone: the built-in "L" LED on pin 13 blinks. It first ran
on 2026-11-13, built by `OKAY\n` at the compiler clone's `4130bf78` plus that day's work in progress.

## Running it

```
okay comp --FinalSafe --target=arduino_uno_r4_wifi blink.cy
arm-none-eabi-objcopy -O binary -j .isr_vector -j .text -j .ARM.exidx -j .data blink.elf blink.bin
arduino-cli upload -p /dev/cu.usbmodem… --fqbn arduino:renesas_uno:unor4wifi --input-file blink.bin
```

`arm-none-eabi-objcopy` and the uploader come with `arduino-cli core install arduino:renesas_uno`. The image starts at
0x4000, after Arduino's bootloader, as the compiler's `memory.ld` places it.

## What was learned

- The LED is the RA4M1's P102, port 1 pin 2. Arduino's bootloader leaves that pin handed to a peripheral, the SPI
  clock, so the port's on/off register is ignored until the pin's own setting, P102PFS, is switched back to plain
  output. That register is write-protected, and unlocked through PWPR.
- The first try did run: halting the chip through the board's CMSIS-DAP debug interface showed it inside the blink
  loop, with the pin set to output. That's how the pin setting was found.
- After using the debug interface, uploads failed with "No device found" until the board was unplugged and plugged back
  in.
- The wait loop's counter compiles to a 128-bit integer, since nothing narrows the range's type, so it takes more
  instructions than it needs to.
