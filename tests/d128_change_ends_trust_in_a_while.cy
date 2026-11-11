// trace: D128, D104

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn.main [none] returns void:
    let mut $s: status set_to status.failed[code set_to 1];
    let mut $n: i32 set_to 2;
    if $s: status.failed:
        while $n above 0:
            let $a: i32 set_to $s.code;
            change $s set_to status.ok;
            change $n set_to $n - 1;
        end while
    end if
    return void;
end fn.main
