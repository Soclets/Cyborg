// trace: D33, D9
import std;
import os;

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn show [$s: status] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    match $s:
        case status.ok:
            std::io::fmt_print(Lend.mut $out, "worked{newln}");
        case status.failed $f:
            std::io::fmt_print(Lend.mut $out, "failed: {}{newln}", $f.code);
    end match
    return void;
end fn show

fn.main [none] returns void:
    show(status.ok);
    show(status.failed[code = 7]);
    return void;
end fn.main
