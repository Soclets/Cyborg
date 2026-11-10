// Two units with no declared unit for their product can't be stored.
// trace: D111, D242

fam.unit length:
    length.meters: base;
end fam.unit length

fam.unit money:
    money.dollars: base;
end fam.unit money

fn.main [none] -> void:
    let $d: i32[unit: length.meters] = 100 length.meters;
    let $m: i32[unit: money.dollars] = 8 money.dollars;
    let $v: i32 = $d x $m;
    return void;
end fn.main
