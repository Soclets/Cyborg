// With no single fix to apply, a knock-on error stays independent.
// trace: D207, D215
@config(default);
import std;

fn.main [] -> void:
    let $a: i32 = totl(5);
    let $b: i32 = $a;
    return void;
end fn.main
