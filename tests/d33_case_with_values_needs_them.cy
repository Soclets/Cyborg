// trace: D33
@config(default);

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [] -> void:
    let $s: status = status.failed;
    return void;
end fn.main
