// trace: D139, D47
import std;

fam.failure file_failed:
    file_failed.not_found;
    file_failed.disk_full[free: i32, label: str];
end fam.failure file_failed

fn save [] -> void:
    fails [file_failed];

    fail file_failed.disk_full[free = 7, label = "scratch"];
end fn save

fn.main [] -> void:
    save() otherwise:
        case not_found:
            std::println("not found");
        case file_failed.disk_full $d:
            std::println(Lend $d.label, " has ", $d.free as str, " free");
    end otherwise
    return void;
end fn.main
