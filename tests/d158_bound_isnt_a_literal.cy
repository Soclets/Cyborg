// trace: D158
@config(default);

const $top: f64.stable = 1.0;

fn.main [] -> void:
    let $t: f64.stable[min: 0.0, max: $top] = 0.5;
    return void;
end fn.main
