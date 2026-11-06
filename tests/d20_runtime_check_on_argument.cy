// trace: D20
fn use_score [$s: i32[min: 0, max: 100]] -> void:
    return void;
end fn use_score

fn.main [] -> void:
    let $n: i32 = -1;
    use_score($n);
    return void;
end fn.main
