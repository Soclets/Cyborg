// `as` can't fail when crossing, and a bound can.
// trace: D141, D20

type.new percent set_to i32[min: 0, max: 100];

fn.main [none] returns void:
    let $n: i32 set_to 150;
    let $p: percent set_to $n as percent;
    return void;
end fn.main
