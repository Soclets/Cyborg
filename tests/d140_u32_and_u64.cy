// D140 brings these two in, and they work like the other integers.
// trace: D140, D42
import std;

fn.main [] -> void:
    let $a: u32 = 4294967295;
    let $b: u64 = 18446744073709551615;
    let $c: u32 = $a plus.wrap 1;
    std::println($a as str);
    std::println($b as str);
    std::println($c as str);
    return void;
end fn.main
