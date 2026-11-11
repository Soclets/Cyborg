// trace: D9

union.tagged status:
    ok;
end union.tagged status

fn.main [none] returns void:
    let $n: i32 = 1;
    if $n: status.ok:
        return void;
    end if
    return void;
end fn.main
