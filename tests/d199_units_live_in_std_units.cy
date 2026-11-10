// trace: D199, D196
import std;

fn.main [none] -> void:
    let $a: i64[unit: std::units::time.seconds.span] = 3600 std::units::time.seconds;
    std::io::fmt_print("{}{newln}", $a);
    return void;
end fn.main
