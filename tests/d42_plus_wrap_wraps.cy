// trace: D42
import std;
@config(default);

fn.main [] -> void:
    let $a: i32 = 2147483647;
    let $b: i32 = $a plus.wrap 1;
    let $c: u8 = 255;
    let $d: u8 = $c plus.wrap 1;
    std::io::fmt_print("{}{newln}", $b);
    std::io::fmt_print("{}{newln}", $d);
    return void;
end fn.main
