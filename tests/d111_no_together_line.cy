// trace: D111
narrowings.unit length:
    length.meters: base;
end narrowings.unit length

narrowings.unit money:
    money.dollars: base;
end narrowings.unit money

fn.main [] -> void:
    let $d: i32.length.meters = 100 length.meters;
    let $m: i32.money.dollars = 8 money.dollars;
    let $v: i32 = $d x $m;
    return void;
end fn.main
