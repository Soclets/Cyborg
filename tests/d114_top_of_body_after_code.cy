// trace: D114

fn half [$n: i32] -> i32:
    let $m: i32 = $n;
    requires $n at_least 0;

    return $m;
end fn half

fn.main [none] -> void:
    return void;
end fn.main
