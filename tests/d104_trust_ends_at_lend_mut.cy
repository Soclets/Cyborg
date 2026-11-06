// trace: D104
union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

fn reset [$s: Lent.mut status] -> void:
    change $s = status.ok;
    return void;
end fn reset

fn.main [] -> void:
    let mut $s: status = status.failed[code = 3];
    if $s: status.failed:
        reset(Lend.mut $s);
        let $b: i32 = $s.code;
    end if
    return void;
end fn.main
