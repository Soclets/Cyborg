// trace: D128

fn.main [none] returns void:
    let $a: f64.stable = 0.5;
    for $i in range[from: $a, to: 1.5]:
        return void;
    end for
    return void;
end fn.main
