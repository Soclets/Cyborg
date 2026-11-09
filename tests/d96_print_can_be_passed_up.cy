// `print_failed` is a placeholder.
// trace: D95, D96
import std;
@config(default);

fn say [] -> void:
    fails [print_failed];

    std::io::fmt_print("hi{newln}")?;
    return void;
end fn say

fn.main [] -> void:
    say();
    return void;
end fn.main
