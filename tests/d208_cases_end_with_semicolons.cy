// trace: D208, D33
@config(default);
import std;

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [] -> void:
    let $s: status = status.failed[code = 3];
    match $s:
        case status.ok:
            std::io::fmt_print("ok{newln}");
        case status.failed $f:
            std::io::fmt_print("{}{newln}", $f.code);
    end match
    return void;
end fn.main
