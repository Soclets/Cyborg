// trace: D126, D9

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let $s: status = status.ok;
    if $s: status.failed:
        let $a: i32 = $s.code;
    else if 1 below 2:
        let $b: i32 = $s.code;
    end if
    return void;
end fn.main
