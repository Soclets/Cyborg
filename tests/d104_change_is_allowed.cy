// trace: D104
import std;
import os;

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $s: status = status.failed[code = 3];
    if $s: status.failed:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $s.code);
        change $s = status.ok;
    end if
    if $s: status.ok:
        std::io::fmt_print(Lend.mut $out, "now ok{newln}");
    end if
    return void;
end fn.main
