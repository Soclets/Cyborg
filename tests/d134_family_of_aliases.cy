// trace: D134, D135
import std;
import os;

fam.type.alias sizes:
    sizes.small = i32;
    sizes.large = i64;
end fam.type.alias sizes

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: sizes.small = 1;
    let $b: i32 = $a;
    let $c: sizes.large = 2;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    let $d: i64 = $c + 1;
    std::io::fmt_print(Lend.mut $out, "large{newln}");
    return void;
end fn.main
