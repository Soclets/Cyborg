// `as` crosses to what it's made from, and an `i32` isn't that.
// trace: D141, D7
type.new user_id = i64;

fn.main [] -> void:
    let $n: i32 = 5;
    let $u: user_id = $n as user_id;
    return void;
end fn.main
