// trace: D134, D135
import std;
import os;

fam.type.alias sizes:
    sizes.small set_to i32;
    sizes.large set_to i64;
end fam.type.alias sizes

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: sizes.small set_to 1;
    let $b: i32 set_to $a;
    let $c: sizes.large set_to 2;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    let $d: i64 set_to $c + 1;
    std::io::fmt_print(Lend.mut $out, "large{newln}");
    return void;
end fn.main
