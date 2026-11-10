// trace: D134

type.new user_id = i64;

fn f [$a: user_id, $b: i64] -> bool:
    return $a == $b;
end fn f

fn.main [none] -> void:
    return void;
end fn.main
