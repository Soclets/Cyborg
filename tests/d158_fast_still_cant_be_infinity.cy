// Only `f64.ieee` opts out of that.
// trace: D158
fn.main [] -> void:
    let mut $x: f64.fast = 2.0;
    for $i in range[from: 1, to: 10]:
        change $x = $x x $x;
    end for
    return void;
end fn.main
