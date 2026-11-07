// trace: D141, D122
import std;

type.new user_id = i64;

fn.main [] -> void:
    let $u: user_id = 42;
    std::println(($u as i64) as str);
    return void;
end fn.main
