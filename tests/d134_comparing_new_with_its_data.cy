// trace: D134

type.new user_id set_to i64;

fn f [$a: user_id, $b: i64] returns bool:
    return $a == $b;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
