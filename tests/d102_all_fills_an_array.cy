// trace: D102
import std;

fn.main [] -> void:
    let $d: std::array<u8>[len: 3 items] = all 255;
    for $x in $d:
        let $copy: u8 = $x;
    end for
    std::println($d.len as str);
    return void;
end fn.main
