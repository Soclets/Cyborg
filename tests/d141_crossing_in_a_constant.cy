// trace: D141, D83
import std;

type.new user_id = i64;

const $root: user_id = 1;
const $plain: i64 = $root as i64;

fn.main [none] -> void:
    std::io::fmt_print("{}{newln}", $plain);
    return void;
end fn.main
