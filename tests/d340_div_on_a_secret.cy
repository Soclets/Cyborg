// div and mod take longer for some values.
// trace: D340
fn.main [none] -> void:
    let $key: u32.secret = 5;
    let $byte: u8.secret = 7;
    let $d: u32.secret = $key div.trunc 2;
    return void;
end fn.main
