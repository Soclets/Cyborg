// trace: D125
@config(default);
import std;

union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

struct job:
    state: status;
end struct job

fn.main [] -> void:
    let mut $j: job = job[state = status.ok];
    change $j.state = status.failed[code = 7];
    match $j.state:
        case status.ok:
            std::io::fmt_print("ok{newln}");
        case status.failed:
            std::io::fmt_print("failed{newln}");
    end match
    return void;
end fn.main
