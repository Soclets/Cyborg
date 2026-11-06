// trace: D125
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
            std::println("ok");
        case status.failed:
            std::println("failed");
    end match
    return void;
end fn.main
