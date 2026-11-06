// How a value gets into a `type.new` isn't decided.
// trace: D134
type.new user_id = i64;

fn.main [] -> void:
    let $u: user_id = 5;
    return void;
end fn.main
