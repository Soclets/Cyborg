// `.` binds to the call first, so this is `??` on a field read, which can't fail.
// trace: D132, D57
struct box:
    n: i32;
end struct box

fn make [] -> box:
    fails [none];

    return box[n = 1];
end fn make

fn.main [] -> void:
    let $a: i32 = make().n ?? 3;
    return void;
end fn.main
