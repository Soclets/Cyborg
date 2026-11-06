// trace: D102
import std;

fn.main [] -> void:
    let $b: std::list<i32>[len.min: 1 items, len.max: 8 items] = [4, 6];
    std::println($b.len as str);
    for $x in $b:
        std::println($x as str);
    end for
    return void;
end fn.main
