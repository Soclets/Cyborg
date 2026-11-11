// trace: D2
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn show [$r: reading] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn show

fn change_it [none] -> void:
    let mut $r: reading = reading[celsius = 21];
    let $b: Lend.mut from $r reading = Lend.mut $r;
    change $b = reading[celsius = 22];
    show(Lend $b);
    return void;
end fn change_it

fn.main [none] -> void:
    change_it();
    return void;
end fn.main
