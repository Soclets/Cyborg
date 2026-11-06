// trace: D42
import std;

fn.main [] -> void:
    let $a: i32 = 2147483647;
    let $b: i32 = $a plus.wrap 1;
    let $c: u8 = 255;
    let $d: u8 = $c plus.wrap 1;
    std::println($b as str);
    std::println($d as str);
    return void;
end fn.main
