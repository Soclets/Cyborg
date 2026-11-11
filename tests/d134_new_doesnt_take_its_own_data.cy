// trace: D134

type.new user_id set_to i64;

fn f [$n: i64] returns void:
    let $u: user_id set_to $n;
    return void;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
