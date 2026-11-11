// trace: D134

type.new user_id = i64;

fn f [$u: user_id] returns void:
    let $n: i64 = $u;
    return void;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
