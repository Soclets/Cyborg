// trace: D49, D116
import std;
@config(default);

fn.main [] -> void:
    let $a: i32 = 6;
    std::println(($a bit_and 3) as str);
    std::println(($a bit_or 1) as str);
    std::println(($a bit_xor 2) as str);
    std::println((bit_not $a) as str);
    std::println(($a shl 2) as str);
    std::println(($a shr 1) as str);
    let $u: u8 = 15;
    let $v: u8 = bit_not $u;
    std::println($v as str);
    let $w: i32 = ($a shl 2) + 1;
    std::println($w as str);
    return void;
end fn.main
