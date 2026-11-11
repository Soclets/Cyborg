// trace: D113, D114

fn add [$a: i32, $b: i32] returns i32:
    labels [$b as to];

    return $a + $b;
end fn add

fn.main [none] returns void:
    let $n: i32 set_to add(1, 2);
    return void;
end fn.main
