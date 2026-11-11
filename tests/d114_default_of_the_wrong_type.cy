// trace: D114, D7

fn f [$a: i32] returns i32:
    labels [$a as first];
    defaults [$a set_to "x"];

    return $a;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
