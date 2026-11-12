// trace: D134, D122

type.new user_id = i64;

fn f [$a: user_id] -> str[len.max: 20 bytes]:
    return $a as str[len.max: 20 bytes];
end fn f

fn.main [none] -> void:
    return void;
end fn.main
