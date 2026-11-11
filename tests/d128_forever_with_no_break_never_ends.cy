// Nothing runs past a `forever` with no `break`, so this needs no `return` after it.
// trace: D128, D11
import std;
import os;

fn first_over [$limit: i32] returns i32:
    let mut $n: i32 = 0;
    forever:
        change $n = $n + 7;
        if $n above $limit:
            return $n;
        end if
    end forever
end fn first_over

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", first_over(10));
    return void;
end fn.main
