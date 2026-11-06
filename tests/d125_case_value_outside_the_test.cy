// trace: D125, D9
union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

fn.main [] -> void:
    let mut $s: status = status.ok;
    change $s.code = 1;
    return void;
end fn.main
