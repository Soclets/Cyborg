// Saturating sticks at the bounds of the type the result goes into, whatever the two values'
// own bounds are, and at the hardware's when that type writes none.
// trace: D294, D372
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: u8[min: 0, max: 100] = 50;
    let $b: u8[min: 0, max: 200] = 150;
    let $c: u8[min: 0, max: 100] = $a plus.saturate $b;
    let $d: u8[min: 0, max: 200] = $a plus.saturate $b;
    let $e: u8 = $b plus.saturate $b;
    let mut $f: u8[min: 10, max: 100] = 60;
    change $f = $a minus.saturate $b;
    std::io::fmt_print(Lend.mut $out, "{} {} {} {}{newln}", $c, $d, $e, $f);
    return void;
end fn.main
