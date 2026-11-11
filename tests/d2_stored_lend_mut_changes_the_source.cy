// trace: D2
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn show [$r: reading] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn show

fn change_it [none] returns void:
    let mut $r: reading set_to reading[celsius set_to 21];
    let $b: Lend.mut from $r reading set_to Lend.mut $r;
    change $b set_to reading[celsius set_to 22];
    show(Lend $b);
    return void;
end fn change_it

fn.main [none] returns void:
    change_it();
    return void;
end fn.main
