// With no single fix to apply, a knock-on error stays independent.
// trace: D207, D215
import std;

fn.main [none] returns void:
    let $a: i32 set_to totl(5);
    let $b: i32 set_to $a;
    return void;
end fn.main
