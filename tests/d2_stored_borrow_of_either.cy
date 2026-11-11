// trace: D2
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: reading = reading[celsius = 1];
    let $c: reading = reading[celsius = 2];
    let $b: Lend from <$a, $c> reading = Lend $c;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.celsius);
    return void;
end fn.main
