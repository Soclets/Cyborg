// `( $x )` is refused.
// trace: D213
import std;

fn.main [none] -> void:
    let $x: i32 = 1;
    let $y: i32 = ( $x ) + 1;
    return void;
end fn.main
