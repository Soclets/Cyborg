// trace: D159, D158
fn.main [] -> void:
    let $a: f64.ieee = 1.5;
    let $b: f64.ieee = 2.5;
    if $a at_most $b:
        return void;
    end if
    return void;
end fn.main
