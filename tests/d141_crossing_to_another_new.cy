// trace: D141, D134
type.new user_id = i64;
type.new order_id = i64;

fn.main [] -> void:
    let $u: user_id = 1;
    let $o: order_id = $u as order_id;
    return void;
end fn.main
