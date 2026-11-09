// trace: D208
@config(default);

union.tagged status:
    ok
    failed[code: i32];
end union.tagged status

fn.main [] -> void:
    return void;
end fn.main
