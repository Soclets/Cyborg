// The word doesn't help, so it's the conversion that gets talked about.
// trace: D149, D7

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn.main [none] returns void:
    let $r: reading = reading[celsius = 1];
    let $n: i32 = $r as i32;
    return void;
end fn.main
