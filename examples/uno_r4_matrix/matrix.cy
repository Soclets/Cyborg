// Scrolls "Merging with the Machine" across the Uno R4 WiFi's 12x8 LED matrix.
// The matrix is 96 LEDs on 11 chip pins, charlieplexed: each LED sits between two pins, and lights
// when one drives high and the other low, so only one is on at a time, and scanning them fast
// makes the picture. Which pins light which LED comes from Arduino's own Arduino_LED_Matrix.
// The tables below are made by a scratch script from a small hand-drawn font. Each column's
// bits 0 to 7 are its 8 rows, top first, and bit 8 is always set, only so that no value is 0:
// today's compiler needs `memset` for long runs of zeros, which its bare-metal runtime lacks.
// No `module` line yet: the compiler is holding modules (D412 to D415) until the tests have them.
// trace: D252, D253, D317
import std;

const $columns: std::coll::array<i32>[len: 145 items] = [
    256, 256, 256, 256, 256, 256, 256, 256,
    256, 256, 256, 256, 383, 258, 268, 258,
    383, 256, 312, 340, 340, 340, 280, 256,
    380, 264, 260, 260, 264, 256, 280, 420,
    420, 420, 380, 256, 381, 256, 380, 264,
    260, 260, 376, 256, 280, 420, 420, 420,
    380, 256, 256, 256, 256, 316, 320, 304,
    320, 316, 256, 381, 256, 260, 319, 324,
    324, 256, 383, 264, 260, 260, 376, 256,
    256, 256, 256, 260, 319, 324, 324, 256,
    383, 264, 260, 260, 376, 256, 312, 340,
    340, 340, 280, 256, 256, 256, 256, 383,
    258, 268, 258, 383, 256, 288, 340, 340,
    340, 376, 256, 312, 324, 324, 324, 288,
    256, 383, 264, 260, 260, 376, 256, 381,
    256, 380, 264, 260, 260, 376, 256, 312,
    340, 340, 340, 280, 256, 256, 256, 256,
    256, 256, 256, 256, 256, 256, 256, 256,
    256
];

const $anodes: std::coll::array<u64>[len: 96 items] = [
    hex 4004_0894, hex 4004_0830, hex 4004_0894, hex 4004_0834, hex 4004_0830, hex 4004_0834, hex 4004_0894, hex 4004_0898,
    hex 4004_0830, hex 4004_0898, hex 4004_0834, hex 4004_0898, hex 4004_0894, hex 4004_080c, hex 4004_0830, hex 4004_080c,
    hex 4004_0834, hex 4004_080c, hex 4004_0898, hex 4004_080c, hex 4004_0894, hex 4004_0890, hex 4004_0830, hex 4004_0890,
    hex 4004_0834, hex 4004_0890, hex 4004_0898, hex 4004_0890, hex 4004_080c, hex 4004_0890, hex 4004_0894, hex 4004_083c,
    hex 4004_0830, hex 4004_083c, hex 4004_0834, hex 4004_083c, hex 4004_0898, hex 4004_083c, hex 4004_080c, hex 4004_083c,
    hex 4004_0890, hex 4004_083c, hex 4004_0894, hex 4004_0810, hex 4004_0830, hex 4004_0810, hex 4004_0834, hex 4004_0810,
    hex 4004_0898, hex 4004_0810, hex 4004_080c, hex 4004_0810, hex 4004_0890, hex 4004_0810, hex 4004_083c, hex 4004_0810,
    hex 4004_0894, hex 4004_082c, hex 4004_0830, hex 4004_082c, hex 4004_0834, hex 4004_082c, hex 4004_0898, hex 4004_082c,
    hex 4004_080c, hex 4004_082c, hex 4004_0890, hex 4004_082c, hex 4004_083c, hex 4004_082c, hex 4004_0810, hex 4004_082c,
    hex 4004_0894, hex 4004_08b4, hex 4004_0830, hex 4004_08b4, hex 4004_0834, hex 4004_08b4, hex 4004_0898, hex 4004_08b4,
    hex 4004_080c, hex 4004_08b4, hex 4004_0890, hex 4004_08b4, hex 4004_083c, hex 4004_08b4, hex 4004_0810, hex 4004_08b4,
    hex 4004_082c, hex 4004_08b4, hex 4004_0894, hex 4004_08b0, hex 4004_0830, hex 4004_08b0, hex 4004_0834, hex 4004_08b0
];

const $cathodes: std::coll::array<u64>[len: 96 items] = [
    hex 4004_0830, hex 4004_0894, hex 4004_0834, hex 4004_0894, hex 4004_0834, hex 4004_0830, hex 4004_0898, hex 4004_0894,
    hex 4004_0898, hex 4004_0830, hex 4004_0898, hex 4004_0834, hex 4004_080c, hex 4004_0894, hex 4004_080c, hex 4004_0830,
    hex 4004_080c, hex 4004_0834, hex 4004_080c, hex 4004_0898, hex 4004_0890, hex 4004_0894, hex 4004_0890, hex 4004_0830,
    hex 4004_0890, hex 4004_0834, hex 4004_0890, hex 4004_0898, hex 4004_0890, hex 4004_080c, hex 4004_083c, hex 4004_0894,
    hex 4004_083c, hex 4004_0830, hex 4004_083c, hex 4004_0834, hex 4004_083c, hex 4004_0898, hex 4004_083c, hex 4004_080c,
    hex 4004_083c, hex 4004_0890, hex 4004_0810, hex 4004_0894, hex 4004_0810, hex 4004_0830, hex 4004_0810, hex 4004_0834,
    hex 4004_0810, hex 4004_0898, hex 4004_0810, hex 4004_080c, hex 4004_0810, hex 4004_0890, hex 4004_0810, hex 4004_083c,
    hex 4004_082c, hex 4004_0894, hex 4004_082c, hex 4004_0830, hex 4004_082c, hex 4004_0834, hex 4004_082c, hex 4004_0898,
    hex 4004_082c, hex 4004_080c, hex 4004_082c, hex 4004_0890, hex 4004_082c, hex 4004_083c, hex 4004_082c, hex 4004_0810,
    hex 4004_08b4, hex 4004_0894, hex 4004_08b4, hex 4004_0830, hex 4004_08b4, hex 4004_0834, hex 4004_08b4, hex 4004_0898,
    hex 4004_08b4, hex 4004_080c, hex 4004_08b4, hex 4004_0890, hex 4004_08b4, hex 4004_083c, hex 4004_08b4, hex 4004_0810,
    hex 4004_08b4, hex 4004_082c, hex 4004_08b0, hex 4004_0894, hex 4004_08b0, hex 4004_0830, hex 4004_08b0, hex 4004_0834
];

fn.main [none] -> void:
    shield matrix:
        reason "the matrix is driven by writing each pin's setting register on the RA4M1"
        // PWPR unlocks the pin setting registers: clear B0WI, then set PFSWE, and leave them open.
        let $protect: ptr.volatile.mut<u8> = ptr.volatile.mut<u8>.at(address: hex 4004_0d03);
        $protect.%write(val: hex 0);
        $protect.%write(val: hex 40);
        forever:
            for $start in range[from: 1, to: 145]:
                for $frame in range[from: 1, to: 40]:
                    for $led in range[from: 0, to: 95]:
                        let $row: i32 = $led div.trunc 12;
                        let $column: i32 = $led mod.trunc 12;
                        let $at: i32 = ($start - 1 + $column) mod.trunc 145;
                        let $bits: i32 = $columns[offset.items: $at];
                        if (($bits shift.right $row) and.bit 1) == 1:
                            let $high: ptr.volatile.mut<u32> = ptr.volatile.mut<u32>.at(address: $anodes[offset.items: $led]);
                            let $low: ptr.volatile.mut<u32> = ptr.volatile.mut<u32>.at(address: $cathodes[offset.items: $led]);
                            // Output, driven high, and output, driven low: PDR is bit 2, PODR bit 0.
                            $high.%write(val: hex 5);
                            $low.%write(val: hex 4);
                            for $wait in range[from: 1, to: 60]:
                                let $seen: u32 = $high.%read();
                            end for
                            $high.%write(val: hex 0);
                            $low.%write(val: hex 0);
                        end if
                    end for
                end for
            end for
        end forever
    end shield matrix
    return void;
end fn.main
