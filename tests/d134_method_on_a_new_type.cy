// It's your own type, so a method can go on it, though nothing can make a value to call it on.
// trace: D134, D130

type.new user_id = i64;

fn user_id.%same [$u: user_id] -> user_id:
    return $u;
end fn user_id.%same

fn f [$u: user_id] -> user_id:
    return $u.%same();
end fn f

fn.main [none] -> void:
    return void;
end fn.main
