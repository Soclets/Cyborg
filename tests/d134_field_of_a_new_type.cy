// trace: D134

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

fn f [$w: wrapped] returns i32:
    return $w.celsius;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
