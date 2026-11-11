// trace: D134

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

fn.main [none] returns void:
    let $w: wrapped set_to wrapped[celsius set_to 1];
    return void;
end fn.main
