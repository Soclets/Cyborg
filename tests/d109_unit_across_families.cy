// `=` can point at a unit in any family.
// trace: D109
import std;
import os;

fam.unit money:
    money.cents: base;
    money.euro_cents: base;
end fam.unit money

fam.unit cash:
    cash.dollars set_to 100 money.cents;
end fam.unit cash

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: i32[unit: cash.dollars] set_to 2 cash.dollars;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    return void;
end fn.main
