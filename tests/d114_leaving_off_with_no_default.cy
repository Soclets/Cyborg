// trace: D114

fn pair [$a: i32, $b: i32] -> i32:
    labels [$b as second];

    return $a;
end fn pair

fn.main [none] -> void:
    let $n: i32 = pair(1);
    return void;
end fn.main
