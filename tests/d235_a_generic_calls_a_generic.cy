// A generic can call another with its own `?item`, lent like any other value, and it needs what the other needs.
// trace: D235, D241
import std;
import os;

interface ordered<?self>:
    op below [$a: Lent ?self, $b: Lent ?self] -> bool;
end interface ordered

interface addable<?self>:
    op + [$a: Lent ?self, $b: Lent ?self] -> ?self;
end interface addable

type_set whole:
    i32;
    i64;
end type_set whole

i32 satisfies ordered;

fn larger<?item: ordered> [$a: Lent ?item, $b: Lent ?item] -> bool:
    return $a above $b;
end fn larger

fn smaller<?thing: ordered> [$a: Lent ?thing, $b: Lent ?thing] -> bool:
    return larger(Lend $b, Lend $a);
end fn smaller

fn shown [$yes: bool] -> str:
    if $yes:
        return "yes";
    end if
    return "no";
end fn shown

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32 = 3;
    let $b: i32 = 8;
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", shown(smaller($a, $b)), shown(smaller($b, $a)));
    return void;
end fn.main
