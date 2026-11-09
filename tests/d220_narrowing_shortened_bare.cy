// A function's narrowing isn't shortened to its bare name, since `stored(…)` alone reads like something else.
// trace: D220, D150
@config(default);
import std;

use std::io::fmt_print.stored as stored;

fn.main [] -> void:
    stored("{}{newln}", ~0.1);
    return void;
end fn.main
