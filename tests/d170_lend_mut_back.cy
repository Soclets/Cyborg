// The borrow points at `$y` itself, so changing through it changes `$y`.
// trace: D170
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

fn colder [$a: Lent.mut reading, $b: Lent.mut reading] returns Lend.mut from <$a, $b> reading:
    if $a.celsius below $b.celsius:
        return Lend.mut $a;
    end if
    return Lend.mut $b;
end fn colder

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $x: reading set_to reading[celsius set_to 21];
    let mut $y: reading set_to reading[celsius set_to 5];
    let $c: Lend.mut from <$x, $y> reading set_to colder(Lend.mut $x, Lend.mut $y);
    change $c.celsius set_to 100;
    std::io::fmt_print(Lend.mut $out, "{} {}{newln}", $x.celsius, $y.celsius);
    return void;
end fn.main
