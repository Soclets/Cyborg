// trace: D169

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [none] returns void:
    let mut $r: reading set_to reading[celsius set_to 21, time set_to 3600];
    let $lock: Lend.held from $r reading set_to Lend $r;
    if $lock.celsius above 0:
        change $r.time set_to 1;
        return void;
    end if
    return void;
end fn.main
