// `as` crosses to what it's made from, and an `i32` isn't that.
// trace: D141, D7

type.new user_id set_to i64;

fn.main [none] returns void:
    let $n: i32 set_to 5;
    let $u: user_id set_to $n as user_id;
    return void;
end fn.main
