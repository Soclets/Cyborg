// trace: D114

fn pair [$a: i32] returns i32:
    labels.in_order [$a as first];

    return $a;
end fn pair

fn.main [none] returns void:
    return void;
end fn.main
