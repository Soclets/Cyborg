// trace: D159

fn.main [none] returns void:
    let mut $x: f64.ieee = 2.0;
    for $i in range[from: 1, to: 10]:
        change $x = $x x $x;
    end for
    let $nan: f64.ieee = $x - $x;
    let $clean: f64.stable = $nan as f64.stable;
    return void;
end fn.main
