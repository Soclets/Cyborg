// trace: D161, D150
import std;

use std::println.stored as stored;
@config(default);

fn.main [] -> void:
    let $x: f64 = 0.5;
    stored($x);
    return void;
end fn.main
