// trace: D174, D2
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn replace [$r: Lent.mut reading] returns void:
    change $r set_to reading[celsius set_to 0];
    return void;
end fn replace

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $r: reading set_to reading[celsius set_to 21];
    let $b: Lend.mut from $r reading set_to Lend.mut $r;
    replace(Lend.mut $b);
    change $b.celsius set_to $b.celsius + 30;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn.main
