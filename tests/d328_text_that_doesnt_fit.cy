// Too much text for a bounded string is a failure, and a bound big enough for every value of
// the type can't fail.
// trace: D328, D326
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $small: i32 = 42;
    let $big: i32 = -2147483648;
    let $all: str[len.max: 11 bytes] = $big as str[len.max: 11 bytes];
    let $fits: str[len.max: 3 bytes] = $small as str[len.max: 3 bytes];
    std::io::fmt_print(Lend.mut $out, "{$all} {$fits}{newln}");
    let $too_much: str[len.max: 3 bytes] = $big as str[len.max: 3 bytes];
    std::io::fmt_print(Lend.mut $out, "{$too_much}{newln}");
    return void;
end fn.main
