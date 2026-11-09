// trace: D199, D196
import std;
@config(default);

fn.main [] -> void:
    let $a: i64.std::units::time.seconds = 3600 std::units::time.seconds;
    std::io::fmt_print("{}{newln}", $a);
    return void;
end fn.main
