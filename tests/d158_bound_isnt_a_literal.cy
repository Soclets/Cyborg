// trace: D158

const $top: f64.stable set_to 1.0;

fn.main [none] returns void:
    let $t: f64.stable[min: 0.0, max: $top] set_to 0.5;
    return void;
end fn.main
