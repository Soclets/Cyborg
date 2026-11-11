// `my::` is your own package's path, so `use` shortens your own types and functions too.
// trace: D248, D150
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn total [$a: i32, $b: i32] -> i32:
    return $a + $b;
end fn total

use my::reading as "r";
use my::total as "sum";

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $x: r = reading[celsius = 4];
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $x.celsius, sum(2, 3));
    return void;
end fn.main
