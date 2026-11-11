// trace: D203
import std;
import os;

fn pick [$hot: bool] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    decl $x: i32;
    if $hot:
        assign $x set_to 1;
    else:
        assign $x set_to 2;
    end if
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $x);
    return void;
end fn pick

fn.main [none] returns void:
    pick(true);
    pick(false);
    return void;
end fn.main
