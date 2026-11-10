// A type's labels go `unit:`, then `min:`, then `max:`, and another order is refused with the right one as its fix.
// trace: D259, D20

fn.main [none] -> void:
    let $n: i32[max: 9, min: 0] = 3;
    return void;
end fn.main
