// trace: D104

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let mut $s: status set_to status.failed[code set_to 3];
    if $s: status.failed:
        let $a: i32 set_to $s.code;
        change $s set_to status.ok;
        let $b: i32 set_to $s.code;
    end if
    return void;
end fn.main
