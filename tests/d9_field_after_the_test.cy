// trace: D9
@config(default);

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [] -> void:
    let $s: status = status.failed[code = 3];
    if $s: status.failed:
        let $n: i32 = $s.code;
    end if
    let $m: i32 = $s.code;
    return void;
end fn.main
