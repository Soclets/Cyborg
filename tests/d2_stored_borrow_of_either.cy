// trace: D2
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: reading set_to reading[celsius set_to 1];
    let $c: reading set_to reading[celsius set_to 2];
    let $b: Lend from <$a, $c> reading set_to Lend $c;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.celsius);
    return void;
end fn.main
