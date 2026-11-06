// trace: D134
type.new user_id = i64;

fn f [$a: user_id, $b: user_id] -> bool:
    return $a == $b;
end fn f

fn.main [] -> void:
    return void;
end fn.main
