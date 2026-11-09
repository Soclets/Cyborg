// trace: D130, D33
@config(default);
import std;

union.tagged status:
    ok;
    failed[code: i32];
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
    std::io::fmt_print("{}{newln}", $a.%code_or_zero());
    std::io::fmt_print("{}{newln}", $b.%code_or_zero());
    return void;
end fn.main
