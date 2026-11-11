// trace: D125, D9

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let mut $s: status set_to status.ok;
    change $s.code set_to 1;
    return void;
end fn.main
