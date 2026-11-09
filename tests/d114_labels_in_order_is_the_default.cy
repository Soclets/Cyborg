// trace: D114
@config(default);

fn pair [$a: i32] -> i32:
    labels.in_order [$a as first];

    return $a;
end fn pair

fn.main [] -> void:
    return void;
end fn.main
