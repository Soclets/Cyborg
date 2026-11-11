// trace: D134, D3

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

fn f [$w: Given wrapped] returns wrapped:
    return $w;
end fn f

fn.main [none] returns void:
    return void;
end fn.main
