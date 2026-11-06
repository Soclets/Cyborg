// trace: D104
union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

fn.main [] -> void:
    let mut $s: status = status.failed[code = 3];
    if $s: status.failed:
        let $a: i32 = $s.code;
        change $s = status.ok;
        let $b: i32 = $s.code;
    end if
    return void;
end fn.main
