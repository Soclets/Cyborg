// `.` binds to the call first, so this is `??` on a field read, which can't fail.
// trace: D132, D57

struct box:
    n: i32;
end struct box

fn make [none] returns box fails [none]:
    return box[n set_to 1];
end fn make

fn.main [none] returns void:
    let $a: i32 set_to make().n ?? 3;
    return void;
end fn.main
