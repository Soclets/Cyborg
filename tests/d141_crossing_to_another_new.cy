// trace: D141, D134

type.new user_id set_to i64;
type.new order_id set_to i64;

fn.main [none] returns void:
    let $u: user_id set_to 1;
    let $o: order_id set_to $u as order_id;
    return void;
end fn.main
