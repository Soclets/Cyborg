// trace: D9

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let $s: status = status.failed[code = 3];
    let $n: i32 = $s.code;
    return void;
end fn.main
