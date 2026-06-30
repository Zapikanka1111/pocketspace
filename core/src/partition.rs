use std::process::Command;

pub fn create_two_partitions(
    disk_number: u32,
    open_size_gb: u64,
    open_letter: char,
    encrypted_letter: char,
) -> Result<(), String> {
    let diskpart_script = format!(
        "select disk {}\n\
         clean\n\
         convert gpt\n\
         create partition primary size={}\n\
         format fs=ntfs quick\n\
         assign letter={}\n\
         create partition primary\n\
         format fs=ntfs quick\n\
         assign letter={}\n",
        disk_number,
        open_size_gb * 1024,
        open_letter,
        encrypted_letter
    );

    let script_path = "C:\\Temp\\diskpart_partitions.txt";
    std::fs::write(script_path, diskpart_script)
        .map_err(|e| format!("Не вдалось записати скрипт: {}", e))?;

    let output = Command::new("diskpart")
        .arg("/s")
        .arg(script_path)
        .output()
        .map_err(|e| format!("Помилка запуску diskpart: {}", e))?;

    println!("DISKPART OUTPUT:\n{}", String::from_utf8_lossy(&output.stdout));

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("diskpart помилка: {}", String::from_utf8_lossy(&output.stderr)))
    }
}