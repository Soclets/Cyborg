// trace: D169
@config(default);

struct reading:
    celsius: i32;
    time: i32;
end struct reading

fn.main [] -> void:
    let mut $r: reading = reading[celsius = 21, time = 3600];
    let $lock: Lend.held from $r reading = Lend $r;
    if $lock.celsius > 0:
        change $r.time = 1;
        return void;
    end if
    return void;
end fn.main
