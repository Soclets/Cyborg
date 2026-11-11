// Inside the block the compiler trusts it, and `else:` isn't inside it, even with two cases.
// trace: D126, D9

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let $s: status set_to status.ok;
    if $s: status.ok:
        return void;
    else:
        let $n: i32 set_to $s.code;
    end if
    return void;
end fn.main
