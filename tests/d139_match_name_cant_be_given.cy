// trace: D139, D2
@config(default);

union.tagged status:
    ok;
    failed[code: i32];
end union.tagged status

fn eat [$s: Given status] -> void:
    return void;
end fn eat

fn.main [] -> void:
    let $s: status = status.failed[code = 1];
    match $s:
        case status.failed $f:
            eat(Give $f);
        case status.ok:
            let $m: i32 = 0;
    end match
    return void;
end fn.main
