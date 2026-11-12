// trace: D130, D9
import std;
import os;

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn status.%label [$s: Lent status] -> str:
    return "a status";
end fn status.%label

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $s: status = status.failed[code = 3];
    if $s: status.failed:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $s.%label());
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $s.code);
    end if
    return void;
end fn.main
