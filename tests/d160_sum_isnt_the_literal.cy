// `==` looks at what's stored, and ~0.1 + ~0.2 doesn't store what ~0.3 does. The linter can flag this.
// trace: D160, D59
import std;
@config(default);

fn.main [] -> void:
    let $a: f64 = ~0.1;
    let $b: f64 = ~0.2;
    let $c: f64 = $a + $b;
    if $c == ~0.3:
        std::println("equal");
    else:
        std::println("different");
    end if
    return void;
end fn.main
