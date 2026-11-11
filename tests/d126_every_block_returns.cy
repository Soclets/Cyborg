// With `else:`, nothing runs past a chain whose blocks all leave.
// trace: D126, D11
import std;
import os;

fn sign [$n: i32] -> i32:
    if $n above 0:
        return 1;
    else if $n below 0:
        return -1;
    else:
        return 0;
    end if
end fn sign

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", sign(5));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", sign(-5));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", sign(0));
    return void;
end fn.main
