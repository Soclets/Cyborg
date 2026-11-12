// `.saturate` on floats sticks at the written bounds of the type the result goes into, even
// where the maths would reach infinity.
// trace: D372, D373
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: f64.stable = 100.0;
    let $b: f64.stable = 50.0;
    let $hot: f64.stable[min: -40.0, max: 125.0] = $a plus.saturate $b;
    let $cold: f64.stable[min: -40.0, max: 125.0] = $b minus.saturate $a;
    let $fine: f64.stable[min: -40.0, max: 125.0] = $a minus.saturate $b;
    let $huge: f64.stable = exponent ~1.0e300;
    let $capped: f64.stable[min: 0.0, max: 1000.0] = $huge times.saturate $huge;
    std::io::fmt_print(Lend.mut $out, "{} {} {} {}{newln}", $hot, $cold, $fine, $capped);
    return void;
end fn.main
