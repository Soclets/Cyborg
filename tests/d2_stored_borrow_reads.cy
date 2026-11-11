// trace: D2
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn read [$r: reading] returns i32:
    return $r.celsius;
end fn read

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $r: reading set_to reading[celsius set_to 21];
    let $b: Lend from $r reading set_to Lend $r;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.celsius);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", read(Lend $b));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn.main
