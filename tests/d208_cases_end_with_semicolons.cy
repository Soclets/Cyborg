// trace: D208, D33
import std;
import os;

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $s: status set_to status.failed[code set_to 3];
    match $s:
        case status.ok:
            std::io::fmt_print(Lend.mut $out, "ok{newln}");
        case status.failed $f:
            std::io::fmt_print(Lend.mut $out, "{}{newln}", $f.code);
    end match
    return void;
end fn.main
