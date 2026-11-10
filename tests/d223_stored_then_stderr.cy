// `.stored` changes what's printed and `.stderr` only where it goes, so `.stored` comes first.
// trace: D221, D223
import std;

fn.main [none] -> void:
    let $tenth: f64.stable = ~0.1;
    std::io::fmt_print.stored.stderr("{}{newln}", $tenth);
    return void;
end fn.main
