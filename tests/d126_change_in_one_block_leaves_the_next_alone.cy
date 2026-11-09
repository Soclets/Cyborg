// The next test runs only if this block didn't, so its `change` can't reach there.
// trace: D126, D104
import std;
@config(default);

union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

fn.main [] -> void:
    let mut $s: status = status.failed[code = 3];
    if $s: status.ok:
        change $s = status.failed[code = 4];
    else if $s: status.failed:
        std::println($s.code as str);
    end if
    return void;
end fn.main
