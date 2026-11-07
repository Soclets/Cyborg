// When every arm leaves, nothing runs past the `match`.
// trace: D9
import std;

union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

fn code_of [$s: status] -> i32:
    match $s:
        case status.ok:
            return 0;
        case status.failed $f:
            return $f.code;
    end match
end fn code_of

fn.main [] -> void:
    std::println(code_of(status.failed[code = 7]) as str);
    return void;
end fn.main
