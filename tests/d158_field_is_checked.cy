// trace: D158, D20

struct gauge:
    ratio: f64.stable[min: 0.0, max: 1.0];
end struct gauge

fn.main [none] returns void:
    let $x: f64.stable set_to 1.5;
    let $g: gauge set_to gauge[ratio set_to $x];
    return void;
end fn.main
