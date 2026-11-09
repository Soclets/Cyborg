// The standard's five prints, with their exact output. Its fourth prints a bare `12.5`, which
// gives NotDesignedYet as a value with no type around it, so this gives it `$price`, which is 12.5.
// trace: D194, D197, D162
@config(default);
import std;

fn.main [] -> void:
    let $name: str = "rope";
    let $count: i32 = 3;
    let $price: f64 = 12.5;
    let $tenth: f64 = ~0.1;
    std::io::fmt_print("hi{newln}");
    std::io::fmt_print("{}: {} at ${places: 2}{newln}", $name, $count, $price);
    std::io::fmt_print("0x{base: hex.upper, zeros, width: 4 cols}{newln}", 255);
    std::io::fmt_print("[{align: right, width: 6 cols}] {form: science, places: 2}{newln}", 42, $price);
    std::io::fmt_print.stored("{}{newln}", $tenth);
    return void;
end fn.main
