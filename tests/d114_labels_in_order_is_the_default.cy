// trace: D114

fn pair [$a: i32] -> i32:
    labels.in_order [$a as first];

    return $a;
end fn pair

fn.main [none] -> void:
    return void;
end fn.main
