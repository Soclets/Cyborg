// trace: D134
@config(default);

type.new user_id = i64;

fn f [$u: user_id, $n: i64] -> void:
    let $v: i64 = $u + $n;
    return void;
end fn f

fn.main [] -> void:
    return void;
end fn.main
