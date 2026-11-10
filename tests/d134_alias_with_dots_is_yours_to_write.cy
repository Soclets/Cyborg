// D10 still holds for the language, so this spelling exists only because this file wrote it.
// trace: D134, D10
import std;

type.alias int.signed.32_bit = i32;

fn.main [none] -> void:
    let $a: int.signed.32_bit = 1;
    let $b: i32 = $a;
    std::io::fmt_print("{}{newln}", $b);
    return void;
end fn.main
