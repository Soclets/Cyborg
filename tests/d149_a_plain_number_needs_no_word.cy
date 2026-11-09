// trace: D149, D123
import std;
@config(default);

type.new user_id = i32;

fn.main [] -> void:
    let $n: i32 = 1;
    let $u: user_id = $n as user_id;
    std::println($n as str);
    return void;
end fn.main
