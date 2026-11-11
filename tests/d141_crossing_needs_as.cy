// trace: D141, D134

type.new user_id set_to i64;

fn.main [none] returns void:
    let $n: i64 set_to 5;
    let $u: user_id set_to $n;
    return void;
end fn.main
