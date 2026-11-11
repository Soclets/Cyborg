// `==` looks at what's stored, and ~0.1 + ~0.2 doesn't store what ~0.3 does. The linter can flag this.
// trace: D160, D59
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: f64.stable = ~0.1;
    let $b: f64.stable = ~0.2;
    let $c: f64.stable = $a + $b;
    if $c == ~0.3:
        std::io::fmt_print(Lend.mut $out, "equal{newln}");
    else:
        std::io::fmt_print(Lend.mut $out, "different{newln}");
    end if
    return void;
end fn.main
