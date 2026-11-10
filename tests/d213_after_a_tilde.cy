// No space follows a `~` on a literal.
// trace: D213
@config(default);
import std;

fn.main [] -> void:
    let $x: f64.stable = ~ 0.1;
    return void;
end fn.main
