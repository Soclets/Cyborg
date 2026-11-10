// Never used again, and still held: that's what the word is for.
// trace: D169

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [none] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $lock: Lend.held from $r reading = Lend $r;
    change $r.celsius = 22;
    return void;
end fn.main
