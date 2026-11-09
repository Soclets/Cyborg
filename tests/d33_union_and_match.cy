// trace: D33, D9
@config(default);
import std;

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn show [$s: status] -> void:
    match $s:
        case status.ok:
            std::io::fmt_print("worked{newln}");
        case status.failed $f:
            std::io::fmt_print("failed: {}{newln}", $f.code);
    end match
    return void;
end fn show

fn.main [] -> void:
    show(status.ok);
    show(status.failed[code = 7]);
    return void;
end fn.main
