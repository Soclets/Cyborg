// trace: D102
import std;
@config(default);

fn.main [] -> void:
    let $b: std::list<i32>[len.min: 1 items, len.max: 8 items] = [4, 6];
    std::io::fmt_print("{}{newln}", $b.len);
    for $x in $b:
        std::io::fmt_print("{}{newln}", $x);
    end for
    return void;
end fn.main
