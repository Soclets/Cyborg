// A held `Lend` only stops changes, so reading `$r` is fine.
// trace: D169
@config(default);
import std;

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $lock: Lend.held from $r reading = Lend $r;
    std::io::fmt_print("{}{newln}", $r.celsius);
    return void;
end fn.main
