// trace: D140
import std;
@config(default);

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items] = [1, 2, 3];
    let $n: u64.items = $xs.len;
    std::io::fmt_print("{}{newln}", $n);
    $xs.std::%append(val: 4);
    std::io::fmt_print("{}{newln}", $xs.len);
    std::io::fmt_print("{}{newln}", $n);
    let $more: u64.items = $xs.len + 2 items;
    std::io::fmt_print("{}{newln}", $more);
    if $more > $n:
        std::io::fmt_print("more{newln}");
    end if
    let $a: std::array<i32>[len: 2 items] = [1, 2];
    let $m: u64.items = $a.len;
    std::io::fmt_print("{}{newln}", $m);
    if $a.len < $xs.len:
        std::io::fmt_print("fewer{newln}");
    end if
    return void;
end fn.main
