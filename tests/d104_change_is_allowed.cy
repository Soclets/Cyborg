// trace: D104
import std;
@config(default);

union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

fn.main [] -> void:
    let mut $s: status = status.failed[code = 3];
    if $s: status.failed:
        std::io::fmt_print("{}{newln}", $s.code);
        change $s = status.ok;
    end if
    if $s: status.ok:
        std::io::fmt_print("now ok{newln}");
    end if
    return void;
end fn.main
