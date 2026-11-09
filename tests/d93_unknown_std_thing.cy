// `std`'s full contents aren't chosen.
// trace: D93
@config(default);
import std;

use std::io::read_line as "readu";

fn.main [] -> void:
    readu();
    return void;
end fn.main
