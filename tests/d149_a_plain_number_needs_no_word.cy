// trace: D149, D123
@config(default);
import std;

type.new user_id = i32;

fn.main [] -> void:
    let $n: i32 = 1;
    let $u: user_id = $n as user_id;
    std::io::fmt_print("{}{newln}", $n);
    return void;
end fn.main
