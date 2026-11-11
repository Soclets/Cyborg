// trace: D134, D122

type.new user_id = i64;

fn f [$a: user_id] returns str:
    return $a as str;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
