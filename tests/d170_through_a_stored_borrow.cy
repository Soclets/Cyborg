// The result names the original value, as D174 says.
// trace: D170, D174
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn first [$a: reading] returns Lend from $a reading:
    return Lend $a;
end fn first

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $x: reading = reading[celsius = 4];
    let $v: Lend from $x reading = Lend $x;
    let $w: Lend from $x reading = first(Lend $v);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $w.celsius);
    return void;
end fn.main
