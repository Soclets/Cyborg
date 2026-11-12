// A literal says its base with a word in front: `hex`, `bin` or `oct`. `_` groups digits in
// any base, and hex digits can be either case, as copied from a datasheet.
// trace: D374
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $address: u64 = hex 4004_0000;
    let $mask: u8 = bin 1111_1110;
    let $mode: u32 = oct 755;
    let $word: u32 = hex DEAD_beef;
    let $low: i32 = -hex 10;
    std::io::fmt_print(Lend.mut $out, "{} {} {} {} {}{newln}", $address, $mask, $mode, $word, $low);
    std::io::fmt_print(Lend.mut $out, "{base: hex}{newln}", $word);
    return void;
end fn.main
