// Plain + checks for overflow, which is a branch, so a secret uses plus.wrap or plus.saturate.
// trace: D341
fn.main [none] -> void:
    let $key: u32.secret = 5;
    let $byte: u8.secret = 7;
    let $a: u32.secret = $key + 1;
    return void;
end fn.main
