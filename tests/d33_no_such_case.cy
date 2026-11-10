// trace: D33

union.tagged status:
    ok;
end union.tagged status

fn.main [none] -> void:
    let $s: status = status.lost;
    return void;
end fn.main
