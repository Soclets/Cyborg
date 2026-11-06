// trace: D130, D33
import std;

union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

fn status.%code_or_zero [$s: status] -> i32:
    if $s: status.failed:
        return $s.code;
    end if
    return 0;
end fn status.%code_or_zero

fn.main [] -> void:
    let $a: status = status.failed[code = 7];
    let $b: status = status.ok;
    std::println($a.%code_or_zero() as str);
    std::println($b.%code_or_zero() as str);
    return void;
end fn.main
