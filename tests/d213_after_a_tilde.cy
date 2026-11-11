// No space follows a `~` on a literal.
// trace: D213
import std;

fn.main [none] -> void:
    let $x: f64.stable = ~ 0.1;
    return void;
end fn.main
