// Handling a call that can't fail isn't decided, and this can't fail.
// trace: D141, D44

type.new user_id set_to i64;

fn.main [none] returns void:
    let $n: i64 set_to 5;
    let $u: user_id set_to $n as user_id ?? 3;
    return void;
end fn.main
