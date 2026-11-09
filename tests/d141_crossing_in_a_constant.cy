// trace: D141, D83
import std;
@config(default);

type.new user_id = i64;

const $root: user_id = 1;
const $plain: i64 = $root as i64;

fn.main [] -> void:
    std::println($plain as str);
    return void;
end fn.main
