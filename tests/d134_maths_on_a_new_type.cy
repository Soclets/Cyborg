// trace: D134
@config(default);

type.new user_id = i64;

fn f [$u: user_id] -> void:
    let $v: user_id = $u + $u;
    return void;
end fn f

fn.main [] -> void:
    return void;
end fn.main
