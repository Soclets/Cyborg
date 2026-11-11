// `$y` changes while `$w` may still point at it.
// trace: D170, D169
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn warmer [$a: reading, $b: reading] -> Lend from <$a, $b> reading:
    if $a.celsius above $b.celsius:
        return Lend $a;
    end if
    return Lend $b;
end fn warmer

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $x: reading = reading[celsius = 21];
    let mut $y: reading = reading[celsius = 25];
    let $w: Lend from <$x, $y> reading = warmer(Lend $x, Lend $y);
    change $y.celsius = 0;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $w.celsius);
    return void;
end fn.main
