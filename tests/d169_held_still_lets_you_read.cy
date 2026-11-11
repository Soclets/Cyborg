// A held `Lend` only stops changes, so reading `$r` is fine.
// trace: D169
import std;
import os;

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $lock: Lend.held from $r reading = Lend $r;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn.main
