// trace: D170
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
    let $y: reading = reading[celsius = 25];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", warmer(Lend $x, Lend $y).celsius);
    return void;
end fn.main
