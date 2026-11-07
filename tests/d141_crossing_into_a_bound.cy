// `as` can't fail when crossing, and a bound can.
// trace: D141, D20
type.new percent = i32[min: 0, max: 100];

fn.main [] -> void:
    let $n: i32 = 150;
    let $p: percent = $n as percent;
    return void;
end fn.main
