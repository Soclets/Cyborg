// Two units with no declared unit for their product can't be stored.
// trace: D111, D242
@config(default);

fam.unit length:
    length.meters: base;
end fam.unit length

fam.unit money:
    money.dollars: base;
end fam.unit money

fn.main [] -> void:
    let $d: i32.length.meters = 100 length.meters;
    let $m: i32.money.dollars = 8 money.dollars;
    let $v: i32 = $d x $m;
    return void;
end fn.main
