// `println` is in the kit part, so a mark on it is still wrong through `use`.
// trace: D155, D22
import @std;

use @std::println as println;
@config(default);

fn.main [] -> void:
    println("hi");
    return void;
end fn.main
