// trace: D29, D84
import std;
import os;

fn right [$n: i32] returns i32:
    ensures result at_least $n;

    return $n + 1;
end fn right

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", right(5));
    return void;
end fn.main
