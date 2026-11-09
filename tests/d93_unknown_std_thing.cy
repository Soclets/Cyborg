// `std`'s full contents aren't chosen.
// trace: D93
import std;

use std::io::read_line as "readu";
@config(default);

fn.main [] -> void:
    readu();
    return void;
end fn.main
