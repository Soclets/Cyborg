// trace: D134, D123
@config(default);

type.new user_id = i64;

fn g [$u: user_id] -> user_id:
    return $u;
end fn g

fn f [$u: user_id] -> user_id:
    return g(Lend $u);
end fn f

fn.main [] -> void:
    return void;
end fn.main
