// trace: D9, D33

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let $s: status set_to status.ok;
    if $s: status.ok:
        let $n: i32 set_to $s.code;
    end if
    return void;
end fn.main
