// trace: D140
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::list<i32>[len.max: inf items] = [1, 2, 3];
    let $n: u64[unit: items] = $xs.len;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    $xs.std::%%append(val: 4);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    let $more: u64[unit: items] = $xs.len + 2 items;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $more);
    if $more above $n:
        std::io::fmt_print(Lend.mut $out, "more{newln}");
    end if
    let $a: std::array<i32>[len: 2 items] = [1, 2];
    let $m: u64[unit: items] = $a.len;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $m);
    if $a.len below $xs.len:
        std::io::fmt_print(Lend.mut $out, "fewer{newln}");
    end if
    return void;
end fn.main
