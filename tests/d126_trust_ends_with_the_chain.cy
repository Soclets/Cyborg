// trace: D126, D9
@config(default);

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [] -> void:
    let $s: status = status.ok;
    if $s: status.failed:
        let $a: i32 = $s.code;
    else if $s: status.ok:
        let $b: i32 = 1;
    end if
    let $c: i32 = $s.code;
    return void;
end fn.main
