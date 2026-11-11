// trace: D9
import std;
import os;

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn show [$s: status] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    if $s: status.failed:
        std::io::fmt_print(Lend.mut $out, "failed: {}{newln}", $s.code);
    end if
    return void;
end fn show

fn.main [none] returns void:
    show(status.failed[code set_to 3]);
    show(status.ok);
    return void;
end fn.main
