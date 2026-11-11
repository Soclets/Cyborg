// `fail` leaves the function too, so the duty has to be over by then.
// trace: D129, D2, D52

struct.must_Give open_file:
    fd: i32;
end struct.must_Give open_file

fn f [none] returns void fails [nope]:
    let $g: open_file = open_file[fd = 3];
    fail nope;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
