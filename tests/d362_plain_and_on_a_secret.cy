// Plain and stops early, which branches, so a secret uses and.both.
// trace: D362
fn.main [none] -> void:
    let $key: u32.secret = 5;
    let $byte: u8.secret = 7;
    let $both: bool.secret = $key == 1 and $key == 2;
    return void;
end fn.main
