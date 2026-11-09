// trace: D114
@config(default);

fn pair [$a: i32] -> i32:
    defaults [$a = 1];

    return $a;
end fn pair

fn.main [] -> void:
    return void;
end fn.main
