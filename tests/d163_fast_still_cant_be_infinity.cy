// Only `f64.ieee` opts out of that.
// trace: D163, D158

fn.main [none] returns void:
    let mut $x: f64.fast set_to 2.0;
    for $i in range[from: 1, to: 10]:
        change $x set_to $x x $x;
    end for
    return void;
end fn.main
