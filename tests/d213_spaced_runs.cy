// Everything spaced the one way.
// trace: D213
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn reading.%shown [$r: reading] returns i32:
    return $r.celsius;
end fn reading.%shown

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: i32 set_to -3;
    let $b: i32 set_to ($a + 10) x 2;
    let $r: reading set_to reading[celsius set_to $b];
    let $near: bool set_to $r.%shown() at_least 0 and $a below 8;
    let $tenth: f64.stable set_to ~0.1;
    if $near:
        std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $r.%shown(), $b div.trunc 4);
    end if
    return void;
end fn.main
