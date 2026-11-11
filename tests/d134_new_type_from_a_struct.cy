// trace: D134

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn.main [none] -> void:
    let $w: wrapped = wrapped[celsius = 1];
    return void;
end fn.main
