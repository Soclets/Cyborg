// trace: D2
import std;
@config(default);

struct reading:
    celsius: i32;
end struct reading

fn show [$r: reading] -> void:
    std::io::fmt_print("{}{newln}", $r.celsius);
    return void;
end fn show

fn change_it [] -> void:
    let mut $r: reading = reading[celsius = 21];
    let $b: Lend.mut from $r reading = Lend.mut $r;
    change $b = reading[celsius = 22];
    show(Lend $b);
    return void;
end fn change_it

fn.main [] -> void:
    change_it();
    return void;
end fn.main
