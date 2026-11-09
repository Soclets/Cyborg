// trace: D151
import std;

use std::println as "say_2";
@config(default);

fn.main [] -> void:
    say_2("hi");
    return void;
end fn.main
