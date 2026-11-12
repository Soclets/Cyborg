// `minus.wrap` and `times.wrap` wrap like `plus.wrap`, in every mode, which settled D42's
// placeholder.
// trace: D295, D42
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $low: i32 = -2147483648;
    let $high: i32 = 2147483647;
    let $byte: u8 = 200;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $high plus.wrap 1);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $low minus.wrap 1);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $byte times.wrap 2);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $high times.wrap $high);
    return void;
end fn.main
