// Handling a call that can't fail isn't decided, and this can't fail.
// trace: D141, D44
type.new user_id = i64;

fn.main [] -> void:
    let $n: i64 = 5;
    let $u: user_id = $n as user_id ?? 3;
    return void;
end fn.main
