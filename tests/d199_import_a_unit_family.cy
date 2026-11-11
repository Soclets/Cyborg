// `import std::units::time;` gives every unit in the family, and nothing else from `std`.
// trace: D199, D198
import std::units::time;

fn.main [none] returns void:
    let $a: i32[unit: std::units::time.seconds.span] = 60 std::units::time.seconds;
    return void;
end fn.main
