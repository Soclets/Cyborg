// trace: D170
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn warmer [$a: Lent reading, $b: Lent reading] -> Lend from <$a, $b> reading:
    if $a.celsius above $b.celsius:
        return Lend $a;
    end if
    return Lend $b;
end fn warmer

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $x: reading = reading[celsius = 21];
    let $y: reading = reading[celsius = 25];
    let $w: Lend from <$y, $x> reading = warmer(Lend $x, Lend $y);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $w.celsius);
    return void;
end fn.main
