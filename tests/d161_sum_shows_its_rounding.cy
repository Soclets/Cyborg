// The shortest text that means this value isn't `0.3`, since `0.3` means something else.
// trace: D161
import std;
@config(default);

fn.main [] -> void:
    let $a: f64 = ~0.1;
    let $b: f64 = ~0.2;
    let $c: f64 = $a + $b;
    std::println($c as str);
    return void;
end fn.main
