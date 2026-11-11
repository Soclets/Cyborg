// trace: D2
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn read [$r: reading] -> i32:
    return $r.celsius;
end fn read

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $r: reading = reading[celsius = 21];
    let $b: Lend from $r reading = Lend $r;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.celsius);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", read(Lend $b));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn.main
