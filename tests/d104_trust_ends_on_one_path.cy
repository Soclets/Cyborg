// trace: D104
@config(default);

union.tagged status:
    ok
    failed[code: i32]
end union.tagged status

fn.main [] -> void:
    let mut $s: status = status.failed[code = 3];
    if $s: status.failed:
        if 1 < 2:
            change $s = status.ok;
        end if
        let $b: i32 = $s.code;
    end if
    return void;
end fn.main
