// trace: D104

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn reset [$s: Lent.mut status] returns void:
    change $s set_to status.ok;
    return void;
end fn reset

fn.main [none] returns void:
    let mut $s: status set_to status.failed[code set_to 3];
    if $s: status.failed:
        reset(Lend.mut $s);
        let $b: i32 set_to $s.code;
    end if
    return void;
end fn.main
