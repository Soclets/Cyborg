// trace: D141, D122
@config(default);
import std;

type.new user_id = i64;

fn.main [] -> void:
    let $u: user_id = 42;
    std::io::fmt_print("{}{newln}", ($u as i64));
    return void;
end fn.main
