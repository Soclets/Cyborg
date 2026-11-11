// trace: D114

fn pair [$a: i32, $b: i32] returns i32:
    labels [$a as first, $b as second];

    return $a;
end fn pair

fn.main [none] returns void:
    let $n: i32 set_to pair(second: 1, first: 2);
    return void;
end fn.main
