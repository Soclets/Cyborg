// trace: D141, D7
@config(default);

type.new user_id = i64;

fn.main [] -> void:
    let $u: user_id = "x";
    return void;
end fn.main
