// A borrow handed back by one call can be handed back again.
// trace: D170
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn warmer [$a: reading, $b: reading] returns Lend from <$a, $b> reading:
    if $a.celsius above $b.celsius:
        return Lend $a;
    end if
    return Lend $b;
end fn warmer

fn warmest [$a: reading, $b: reading] returns Lend from <$a, $b> reading:
    return warmer(Lend $a, Lend $b);
end fn warmest

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $x: reading set_to reading[celsius set_to 21];
    let $y: reading set_to reading[celsius set_to 25];
    let $w: Lend from <$x, $y> reading set_to warmest(Lend $x, Lend $y);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $w.celsius);
    return void;
end fn.main
