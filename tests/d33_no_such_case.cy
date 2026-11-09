// trace: D33
@config(default);

union.tagged status:
    ok
end union.tagged status

fn.main [] -> void:
    let $s: status = status.lost;
    return void;
end fn.main
