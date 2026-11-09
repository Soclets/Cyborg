// trace: D102
import std;
@config(default);

fn.main [] -> void:
    let $d: std::array<u8>[len: 3 items] = all 255;
    for $x in $d:
        let $copy: u8 = $x;
    end for
    std::io::fmt_print("{}{newln}", $d.len);
    return void;
end fn.main
