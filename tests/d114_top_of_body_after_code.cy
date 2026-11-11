// trace: D114

fn half [$n: i32] returns i32:
    let $m: i32 set_to $n;
    requires $n at_least 0;

    return $m;
end fn half

fn.main [none] returns void:
    return void;
end fn.main
