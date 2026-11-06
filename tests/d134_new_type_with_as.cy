// trace: D134, D7
type.new user_id = i64;

fn.main [] -> void:
    let $n: i64 = 5;
    let $u: user_id = $n as user_id;
    return void;
end fn.main
