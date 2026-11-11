// The path that doesn't assign returns, so every path that reaches the read has assigned.
// trace: D203
import std;
import os;

fn pick [$hot: bool] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    decl $x: i32;
    if $hot:
        assign $x = 1;
    else:
        return void;
    end if
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
    return void;
end fn pick

fn.main [none] -> void:
    pick(true);
    return void;
end fn.main
