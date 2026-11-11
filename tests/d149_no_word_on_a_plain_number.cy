// A plain number is always copied, so it takes no word here either.
// trace: D149, D123

type.new user_id set_to i32;

fn.main [none] returns void:
    let $n: i32 set_to 1;
    let $u: user_id set_to Copy $n as user_id;
    return void;
end fn.main
