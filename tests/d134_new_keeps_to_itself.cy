// Handing one along as itself is fine: it's only other types it doesn't mix with.
// trace: D134, D123

type.new user_id = i64;

fn g [$u: user_id] returns user_id:
    let $same: user_id = $u;
    return $same;
end fn g

fn f [$u: user_id] returns user_id:
    return g($u);
end fn f

fn.main [none] returns void:
    return void;
end fn.main
