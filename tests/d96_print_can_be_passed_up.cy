// `print_failed` is a placeholder.
// trace: D95, D96
import std;

fn say [none] -> void fails [print_failed]:
    std::io::fmt_print("hi{newln}")?;
    return void;
end fn say

fn.main [none] -> void:
    say();
    return void;
end fn.main
