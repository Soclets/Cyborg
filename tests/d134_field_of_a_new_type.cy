// trace: D134
@config(default);

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn f [$w: wrapped] -> i32:
    return $w.celsius;
end fn f

fn.main [] -> void:
    return void;
end fn.main
