// trace: D52, D96
@config(default);
import std;

fn half [$n: i32] -> i32:
    fails [odd];

    if $n mod.trunc 2 != 0:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", half(4));
    std::io::fmt_print("{}{newln}", half(3));
    std::io::fmt_print("not reached{newln}");
    return void;
end fn.main
