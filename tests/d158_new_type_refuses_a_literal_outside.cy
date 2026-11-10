// trace: D158, D141
@config(default);

type.new ratio = f64.stable[min: 0.0, max: 1.0];

fn.main [] -> void:
    let $r: ratio = 1.5;
    return void;
end fn.main
