// trace: D126, D9

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let $s: status set_to status.ok;
    if $s: status.failed:
        let $a: i32 set_to $s.code;
    else if 1 below 2:
        let $b: i32 set_to $s.code;
    end if
    return void;
end fn.main
