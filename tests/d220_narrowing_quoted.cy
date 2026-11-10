// A quoted name for a function's narrowing is your own choice.
// trace: D220, D150
@config(default);
import std;

use std::io::fmt_print.stored as "print_exact";

fn.main [] -> void:
    let $tenth: f64.stable = ~0.1;
    print_exact("{}{newln}", $tenth);
    return void;
end fn.main
