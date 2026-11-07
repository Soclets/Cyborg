// The oracle stays plain, so `f64.fast` runs just as written here.
// trace: D158, D62
import std;

fn.main [] -> void:
    let $a: f64.fast = 1.5;
    let $b: f64.fast = $a + $a x 2.0;
    std::println("ran");
    return void;
end fn.main
