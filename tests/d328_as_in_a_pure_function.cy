// `as` never allocates, so a pure function can turn a number into bounded text.
// trace: D328, D86
import std;
import os;

fn.pure f [$n: i32] -> str[len.max: 11 bytes]:
    return $n as str[len.max: 11 bytes];
end fn.pure f

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", f(-2147483648));
    return void;
end fn.main
