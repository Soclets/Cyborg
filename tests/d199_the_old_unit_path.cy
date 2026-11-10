// trace: D199
@config(default);
import std;

fn.main [] -> void:
    let $a: i64[unit: std::time.seconds] = 3600 std::time.seconds;
    return void;
end fn.main
