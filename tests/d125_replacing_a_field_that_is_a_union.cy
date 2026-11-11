// trace: D125
import std;
import os;

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

struct job:
    state: status;
end struct job

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $j: job = job[state = status.ok];
    change $j.state = status.failed[code = 7];
    match $j.state:
        case status.ok:
            std::io::fmt_print(Lend.mut $out, "ok{newln}");
        case status.failed:
            std::io::fmt_print(Lend.mut $out, "failed{newln}");
    end match
    return void;
end fn.main
