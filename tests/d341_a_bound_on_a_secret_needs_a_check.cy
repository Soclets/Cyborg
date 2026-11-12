// A bound on a secret needs a check, which branches, unless it's proven away.
// trace: D341
fn.main [none] -> void:
    let $key: u32.secret = 5;
    let $byte: u8.secret = 7;
    let $small: u8.secret[min: 0, max: 100] = $byte plus.wrap 1;
    return void;
end fn.main
