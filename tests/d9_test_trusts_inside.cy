// trace: D9
import std;

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn show [$s: status] -> void:
    if $s: status.failed:
        std::io::fmt_print("failed: {}{newln}", $s.code);
    end if
    return void;
end fn show

fn.main [none] -> void:
    show(status.failed[code = 3]);
    show(status.ok);
    return void;
end fn.main
