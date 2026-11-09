// Crossing is the same data, so it isn't the allocating kind of `as`.
// trace: D141, D86
import std;
@config(default);

type.new user_id = i64;

fn.pure raw [$u: user_id] -> i64:
    return $u as i64;
end fn.pure raw

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", raw(3));
    return void;
end fn.main
