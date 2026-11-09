// trace: D134, D122
@config(default);

type.new user_id = i64;

fn f [$a: user_id] -> str:
    return $a as str;
end fn f

fn.main [] -> void:
    return void;
end fn.main
