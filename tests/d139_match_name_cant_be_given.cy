// trace: D139, D2

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn eat [$s: Given status] returns void:
    return void;
end fn eat

fn.main [none] returns void:
    let $s: status set_to status.failed[code set_to 1];
    match $s:
        case status.failed $f:
            eat(Give $f);
        case status.ok:
            let $m: i32 set_to 0;
    end match
    return void;
end fn.main
