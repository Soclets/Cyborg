// `small` is its own struct here, so the family's `sizes.small` doesn't come into it.
// trace: D134, D57
import std;
import os;

struct small:
    n: i32;
end struct small

fam.type.alias sizes:
    sizes.small set_to i32;
end fam.type.alias sizes

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: small set_to small[n set_to 1];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a.n);
    return void;
end fn.main
