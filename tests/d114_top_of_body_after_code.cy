// trace: D114
@config(default);

fn half [$n: i32] -> i32:
    let $m: i32 = $n;
    requires $n at_least 0;

    return $m;
end fn half

fn.main [] -> void:
    return void;
end fn.main
