// trace: D128, D104

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] -> void:
    let mut $s: status = status.failed[code = 1];
    let mut $n: i32 = 2;
    if $s: status.failed:
        while $n above 0:
            let $a: i32 = $s.code;
            change $s = status.ok;
            change $n = $n - 1;
        end while
    end if
    return void;
end fn.main
