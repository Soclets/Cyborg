// trace: D140
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $xs: std::coll::list<i32>[len.max: inf items] = [1, 2, 3];
    defer std::coll::list.done(Give $xs, alloc: Lend $heap);
    let $n: u64[unit: items] = $xs.len;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    $xs.std::%%append(val: 4, alloc: Lend $heap);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
    let $more: u64[unit: items] = $xs.len + 2 items;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $more);
    if $more above $n:
        std::io::fmt_print(Lend.mut $out, "more{newln}");
    end if
    let $a: std::coll::array<i32>[len: 2 items] = [1, 2];
    let $m: u64[unit: items] = $a.len;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $m);
    if $a.len below $xs.len:
        std::io::fmt_print(Lend.mut $out, "fewer{newln}");
    end if
    return void;
end fn.main
