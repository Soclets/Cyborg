// trace: D130, D33
import std;
import os;

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn status.%code_or_zero [$s: status] returns i32:
    if $s: status.failed:
        return $s.code;
    end if
    return 0;
end fn status.%code_or_zero

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: status set_to status.failed[code set_to 7];
    let $b: status set_to status.ok;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a.%code_or_zero());
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b.%code_or_zero());
    return void;
end fn.main
