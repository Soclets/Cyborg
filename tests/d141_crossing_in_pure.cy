// Crossing is the same data, so it isn't the allocating kind of `as`.
// trace: D141, D86
import std;
@config(default);

type.new user_id = i64;

fn.pure raw [$u: user_id] -> i64:
    return $u as i64;
end fn.pure raw

fn.main [] -> void:
    std::println(raw(3) as str);
    return void;
end fn.main
