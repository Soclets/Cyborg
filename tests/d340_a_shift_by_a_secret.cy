// Some CPUs take longer for bigger shifts, so a shift's amount stays public.
// trace: D340
fn.main [none] -> void:
    let $key: u32.secret = 5;
    let $byte: u8.secret = 7;
    let $s: u32.secret = 1 shl $key;
    return void;
end fn.main
