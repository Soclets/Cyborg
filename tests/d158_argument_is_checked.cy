// trace: D158, D20

fn use_ratio [$r: f64.stable[min: 0.0, max: 1.0]] returns void:
    return void;
end fn use_ratio

fn.main [none] returns void:
    let $x: f64.stable set_to 1.5;
    use_ratio($x);
    return void;
end fn.main
