// trace: D126, D9

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let $s: status set_to status.ok;
    if $s: status.failed:
        let $a: i32 set_to $s.code;
    else if $s: status.ok:
        let $b: i32 set_to 1;
    end if
    let $c: i32 set_to $s.code;
    return void;
end fn.main
