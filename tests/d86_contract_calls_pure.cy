// trace: D86
import std;
import os;

fn.pure positive [$n: i32] returns bool:
    return $n at_least 0;
end fn.pure positive

fn f [$n: i32] returns i32:
    requires positive($n);

    return $n;
end fn f

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", f(1));
    return void;
end fn.main
