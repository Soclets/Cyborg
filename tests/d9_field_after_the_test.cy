// trace: D9

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let $s: status set_to status.failed[code set_to 3];
    if $s: status.failed:
        let $n: i32 set_to $s.code;
    end if
    let $m: i32 set_to $s.code;
    return void;
end fn.main
