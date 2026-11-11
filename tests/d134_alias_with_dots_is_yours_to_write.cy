// D10 still holds for the language, so this spelling exists only because this file wrote it.
// trace: D134, D10
import std;
import os;

type.alias int.signed.32_bit = i32;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: int.signed.32_bit = 1;
    let $b: i32 = $a;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    return void;
end fn.main
