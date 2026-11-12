// Everything spaced the one way.
// trace: D213
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn reading.%shown [$r: Lent reading] -> i32:
    return $r.celsius;
end fn reading.%shown

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32 = -3;
    let $b: i32 = ($a + 10) x 2;
    let $r: reading = reading[celsius = $b];
    let $near: bool = $r.%shown() at_least 0 and $a below 8;
    let $tenth: f64.stable = ~0.1;
    if $near:
        std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $r.%shown(), $b div.trunc 4);
    end if
    return void;
end fn.main
