// trace: D134

type.new user_id = i64;
type.new order_id = i64;

fn f [$u: user_id] -> void:
    let $o: order_id = $u;
    return void;
end fn f

fn.main [none] -> void:
    return void;
end fn.main
