// trace: D134

type.new user_id set_to i64;

fn f [$u: user_id, $n: i64] returns void:
    let $v: i64 set_to $u + $n;
    return void;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
