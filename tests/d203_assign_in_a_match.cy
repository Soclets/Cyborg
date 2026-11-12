// trace: D203, D9
import std;
import os;

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn code_of [$s: Lent status] -> i32:
    decl $n: i32;
    match $s:
        case status.ok:
            assign $n = 0;
        case status.failed $f:
            assign $n = $f.code;
    end match
    return $n;
end fn code_of

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", code_of(status.failed[code = 7]));
    return void;
end fn.main
