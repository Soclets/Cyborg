// trace: D104
import std;

union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

fn.main [] -> void:
    let mut $s: status = status.failed[code = 3];
    if $s: status.failed:
        std::println($s.code as str);
        change $s = status.ok;
    end if
    if $s: status.ok:
        std::println("now ok");
    end if
    return void;
end fn.main
