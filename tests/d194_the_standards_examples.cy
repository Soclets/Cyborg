// The standard's five prints, with their exact output.
// trace: D194, D197, D162
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $name: str = "rope";
    let $count: i32 = 3;
    let $price: f64.stable = 12.5;
    let $tenth: f64.stable = ~0.1;
    std::io::fmt_print(Lend.mut $out, "hi{newln}");
    std::io::fmt_print(Lend.mut $out, "{}: {} at ${places: 2}{newln}", $name, $count, $price);
    std::io::fmt_print(Lend.mut $out, "0x{base: hex.upper, zeros, width: 4 cols}{newln}", 255);
    std::io::fmt_print(Lend.mut $out, "[{align: right, width: 6 cols}] {form: science, places: 2}{newln}", 42, $price);
    std::io::fmt_print.stored(Lend.mut $out, "{}{newln}", $tenth);
    return void;
end fn.main
