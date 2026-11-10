// A generic can call another with its own `?item`, lent like any other value, and it needs what the other needs.
// trace: D235, D241
@config(default);
import std;

interface ordered<?self>:
    op < [$a: ?self, $b: ?self] -> bool;
end interface ordered

interface addable<?self>:
    op + [$a: ?self, $b: ?self] -> ?self;
end interface addable

type_set whole:
    i32;
    i64;
end type_set whole

i32 satisfies ordered;

fn larger<?item: ordered> [$a: ?item, $b: ?item] -> bool:
    return $a > $b;
end fn larger

fn smaller<?thing: ordered> [$a: ?thing, $b: ?thing] -> bool:
    return larger(Lend $b, Lend $a);
end fn smaller

fn shown [$yes: bool] -> str:
    if $yes:
        return "yes";
    end if
    return "no";
end fn shown

fn.main [] -> void:
    let $a: i32 = 3;
    let $b: i32 = 8;
    std::io::fmt_print("{} {}{newln}", shown(smaller($a, $b)), shown(smaller($b, $a)));
    return void;
end fn.main
