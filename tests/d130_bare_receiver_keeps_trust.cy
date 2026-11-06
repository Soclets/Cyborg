// trace: D130, D9
import std;

union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

fn status.%label [$s: status] -> str:
    return "a status";
end fn status.%label

fn.main [] -> void:
    let $s: status = status.failed[code = 3];
    if $s: status.failed:
        std::println($s.%label());
        std::println($s.code as str);
    end if
    return void;
end fn.main
