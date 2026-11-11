// 1.14's example: a long call broken after its commas, with its closing bracket on its own line.
// trace: D219
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $name: str = "rope";
    let $count: i32 = 3;
    let $price: f64.stable = 12.5;
    std::io::fmt_print(
        Lend.mut $out,
        "{}: {} at ${places: 2}{newln}",
        $name,
        $count,
        $price
    );
    return void;
end fn.main
