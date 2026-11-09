// trace: D114, D7
@config(default);

fn f [$a: i32] -> i32:
    labels [$a as first];
    defaults [$a = "x"];

    return $a;
end fn f

fn.main [] -> void:
    return void;
end fn.main
