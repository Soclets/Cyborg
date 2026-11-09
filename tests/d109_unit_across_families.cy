// `=` can point at a unit in any family.
// trace: D109
@config(default);
import std;

fam.unit money:
    money.cents: base;
    money.euro_cents: base;
end fam.unit money

fam.unit cash:
    cash.dollars = 100 money.cents;
end fam.unit cash

fn.main [] -> void:
    let $a: i32.cash.dollars = 2 cash.dollars;
    std::io::fmt_print("{}{newln}", $a);
    return void;
end fn.main
