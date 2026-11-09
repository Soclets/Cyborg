// trace: D130, D104
@config(default);

union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

fn status.%reset [$s: Lent.mut status] -> void:
    change $s = status.ok;
    return void;
end fn status.%reset

fn.main [] -> void:
    let mut $s: status = status.failed[code = 3];
    if $s: status.failed:
        $s.%reset();
        let $n: i32 = $s.code;
    end if
    return void;
end fn.main
