// `( $x )` is refused.
// trace: D213
import std;

fn.main [none] returns void:
    let $x: i32 set_to 1;
    let $y: i32 set_to ( $x ) + 1;
    return void;
end fn.main
