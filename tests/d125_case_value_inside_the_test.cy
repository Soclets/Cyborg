// D125 speaks of fields, and whether the test's trust survives this isn't stated.
// trace: D125, D104
@config(default);

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [] -> void:
    let mut $s: status = status.failed[code = 1];
    if $s: status.failed:
        change $s.code = 2;
    end if
    return void;
end fn.main
