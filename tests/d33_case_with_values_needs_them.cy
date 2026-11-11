// trace: D33

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let $s: status = status.failed;
    return void;
end fn.main
