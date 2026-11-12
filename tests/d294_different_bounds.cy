// Saturating two values whose bounds differ isn't decided: which bounds would it stick at?
// trace: D294
fn.main [none] -> void:
    let $a: u8[min: 0, max: 100] = 50;
    let $b: u8[min: 0, max: 200] = 150;
    let $c: u8 = $a plus.saturate $b;
    return void;
end fn.main
