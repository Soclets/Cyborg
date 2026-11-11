// The word doesn't help, so it's the conversion that gets talked about.
// trace: D149, D7

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

fn.main [none] returns void:
    let $r: reading set_to reading[celsius set_to 1];
    let $n: i32 set_to $r as i32;
    return void;
end fn.main
