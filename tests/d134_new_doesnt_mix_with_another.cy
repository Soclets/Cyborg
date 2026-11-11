// trace: D134

type.new user_id set_to i64;
type.new order_id set_to i64;

fn f [$u: user_id] returns void:
    let $o: order_id set_to $u;
    return void;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
