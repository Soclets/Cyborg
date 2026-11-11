// trace: D114

fn half [$n: i32] -> i32:
    ensures result at_least 0;
    requires $n at_least 0;
    return $n;
end fn half

fn.main [none] -> void:
    return void;
end fn.main
