// trace: D134, D123

type.new user_id set_to i64;

fn g [$u: user_id] returns user_id:
    return $u;
end fn g

fn f [$u: user_id] returns user_id:
    return g(Lend $u);
end fn f

fn.main [none] returns void:
    return void;
end fn.main
