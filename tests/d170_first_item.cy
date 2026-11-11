// The standard's example: lending an item names the whole list.
// trace: D170, D172
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn first [$xs: std::list<reading>[len.min: 1 items, len.max: inf items]] returns Lend from $xs reading:
    return Lend $xs[pos: 1];
end fn first

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $xs: std::list<reading>[len.min: 1 items, len.max: inf items] set_to [reading[celsius set_to 3]];
    let $f: Lend from $xs reading set_to first(Lend $xs);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $f.celsius);
    return void;
end fn.main
