// A saturating operator sticks at the type's own bounds: its written `min:` and `max:`, or
// the hardware's on a plain type.
// trace: D294, D295
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $volume: u8[min: 0, max: 100] = 95;
    let $louder: u8[min: 0, max: 100] = $volume plus.saturate 10;
    let $quieter: u8[min: 0, max: 100] = $volume minus.saturate 200;
    let $byte: u8 = 250;
    let $high: i32 = 2147483647;
    let $low: i32 = -2147483648;
    std::io::fmt_print(Lend.mut $out, "{$louder} {$quieter}{newln}");
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $byte plus.saturate 10);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $byte minus.saturate 251);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $high times.saturate 2);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $low times.saturate 2);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $high plus.saturate -1);
    return void;
end fn.main
