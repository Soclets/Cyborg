// trace: D208, D33
import std;
import os;

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $s: status = status.failed[code = 3];
    match $s:
        case status.ok:
            std::io::fmt_print(Lend.mut $out, "ok{newln}");
        case status.failed $f:
            std::io::fmt_print(Lend.mut $out, "{}{newln}", $f.code);
    end match
    return void;
end fn.main
