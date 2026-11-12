// `Lent` lends a struct to read, and the call writes `Lend`.
// trace: D393, D2
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn show [$r: Lent reading, $by: i32] -> i32:
    return $r.celsius + $by;
end fn show

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $r: reading = reading[celsius = 21];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", show(Lend $r, 2));
    return void;
end fn.main
