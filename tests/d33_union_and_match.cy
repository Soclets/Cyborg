// trace: D33, D9
import std;

union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

fn show [$s: status] -> void:
    match $s:
        case status.ok:
            std::println("worked");
        case status.failed:
            std::println("failed: ", $s.code as str);
    end match
    return void;
end fn show

fn.main [] -> void:
    show(status.ok);
    show(status.failed[code = 7]);
    return void;
end fn.main
