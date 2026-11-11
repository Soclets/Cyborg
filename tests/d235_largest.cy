// 5.8's `largest`, checked once, then called with `i32` and with a struct of your own. Its
// `return` says `Copy`, since a type parameter is handed off like a struct is.
// trace: D234, D235, D238, D241, D236
import std;
import os;

interface equatable<?self>:
    op == [$a: ?self, $b: ?self] -> bool;
end interface equatable

interface ordered<?self>:
    includes equatable;
    op < [$a: ?self, $b: ?self] -> bool;
    law if $a < $b and $b < $c then $a < $c;
end interface ordered

struct reading:
    celsius: i32;
end struct reading

op < [$a: reading, $b: reading] -> bool:
    return $a.celsius < $b.celsius;
end op <

op == [$a: reading, $b: reading] -> bool:
    return $a.celsius == $b.celsius;
end op ==

i32 satisfies ordered;
reading .|. ordered;

fn largest<?item: ordered> [$a: ?item, $b: ?item] -> ?item:
    if $a > $b:
        return Copy $a;
    end if
    return Copy $b;
end fn largest

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $x: i32 = largest(3, 5);
    let $y: i32 = 9;
    let $z: i32 = 4;
    let $cold: reading = reading[celsius = 3];
    let $warm: reading = reading[celsius = 21];
    let $top: reading = largest(Lend $cold, Lend $warm);
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", $x, largest($y, $z), $top.celsius);
    return void;
end fn.main
