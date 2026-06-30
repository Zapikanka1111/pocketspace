use windows::{
    core::HSTRING,
    Win32::Foundation::{HANDLE, NO_ERROR},
    Win32::Storage::Vhd::{
        CreateVirtualDisk, OpenVirtualDisk, AttachVirtualDisk, DetachVirtualDisk,
        GetVirtualDiskPhysicalPath,
        CREATE_VIRTUAL_DISK_FLAG_NONE,
        CREATE_VIRTUAL_DISK_PARAMETERS, OPEN_VIRTUAL_DISK_PARAMETERS,
        OPEN_VIRTUAL_DISK_FLAG, ATTACH_VIRTUAL_DISK_FLAG, DETACH_VIRTUAL_DISK_FLAG,
        OPEN_VIRTUAL_DISK_VERSION_2,
        VIRTUAL_DISK_ACCESS_ALL, VIRTUAL_DISK_ACCESS_NONE,
        VIRTUAL_STORAGE_TYPE_DEVICE_VHD, VIRTUAL_STORAGE_TYPE,
        VIRTUAL_STORAGE_TYPE_VENDOR_MICROSOFT,
    },
};

pub fn get_physical_path(handle: HANDLE) -> Result<String, String> {
    let mut buffer = vec![0u16; 256];
    let mut size: u32 = (buffer.len() * 2) as u32;

    let result = unsafe {
        GetVirtualDiskPhysicalPath(handle, &mut size, windows::core::PWSTR(buffer.as_mut_ptr()))
    };

    if result != NO_ERROR {
        return Err(format!("Помилка отримання шляху диску: {:?}", result));
    }

    let path = String::from_utf16_lossy(&buffer);
    let path = path.trim_end_matches('\0').to_string();
    Ok(path)
}

pub fn create_vhd(path: &str, size_gb: u64) -> Result<(), String> {
    let size_bytes = size_gb * 1024 * 1024 * 1024;

    let storage_type = VIRTUAL_STORAGE_TYPE {
        DeviceId: VIRTUAL_STORAGE_TYPE_DEVICE_VHD,
        VendorId: VIRTUAL_STORAGE_TYPE_VENDOR_MICROSOFT,
    };

    let mut params = CREATE_VIRTUAL_DISK_PARAMETERS::default();
    params.Version = windows::Win32::Storage::Vhd::CREATE_VIRTUAL_DISK_VERSION_1;
    params.Anonymous.Version1.MaximumSize = size_bytes;
    params.Anonymous.Version1.BlockSizeInBytes = 0;
    params.Anonymous.Version1.SectorSizeInBytes = 512;

    let path_hstring = HSTRING::from(path);
    let mut handle = HANDLE::default();

    let result = unsafe {
        CreateVirtualDisk(
            &storage_type,
            &path_hstring,
            VIRTUAL_DISK_ACCESS_ALL,
            None,
            CREATE_VIRTUAL_DISK_FLAG_NONE,
            0,
            &params,
            None,
            &mut handle,
        )
    };

    println!("DEBUG: result = {:?}, code = {}", result, result.0);

    if result == NO_ERROR {
        Ok(())
    } else {
        Err(format!("Помилка створення VHD: {:?}", result))
    }
}

pub fn mount_and_get_handle(path: &str) -> Result<HANDLE, String> {
    let storage_type = VIRTUAL_STORAGE_TYPE {
        DeviceId: VIRTUAL_STORAGE_TYPE_DEVICE_VHD,
        VendorId: VIRTUAL_STORAGE_TYPE_VENDOR_MICROSOFT,
    };

    let path_hstring = HSTRING::from(path);
    let mut handle = HANDLE::default();

    let mut open_params = OPEN_VIRTUAL_DISK_PARAMETERS::default();
    open_params.Version = OPEN_VIRTUAL_DISK_VERSION_2;
    open_params.Anonymous.Version2.GetInfoOnly = false.into();
    open_params.Anonymous.Version2.ReadOnly = false.into();

    let open_result = unsafe {
        OpenVirtualDisk(
            &storage_type,
            &path_hstring,
            VIRTUAL_DISK_ACCESS_NONE,
            OPEN_VIRTUAL_DISK_FLAG(0),
            Some(&open_params),
            &mut handle,
        )
    };

    if open_result != NO_ERROR {
        return Err(format!("Помилка відкриття VHD: {:?}", open_result));
    }

    let attach_result = unsafe {
        AttachVirtualDisk(handle, None, ATTACH_VIRTUAL_DISK_FLAG(0), 0, None, None)
    };

    if attach_result != NO_ERROR {
        return Err(format!("Помилка монтування VHD: {:?}", attach_result));
    }

    Ok(handle)
}

pub fn detach_with_handle(handle: HANDLE) -> Result<(), String> {
    let detach_result = unsafe {
        DetachVirtualDisk(handle, DETACH_VIRTUAL_DISK_FLAG(0), 0)
    };

    if detach_result == NO_ERROR {
        Ok(())
    } else {
        Err(format!("Помилка демонтування VHD: {:?}", detach_result))
    }
}

use std::process::Command;

pub fn format_vhd(handle: HANDLE, drive_letter: char) -> Result<(), String> {
    let physical_path = get_physical_path(handle)?;
    
    let disk_number: u32 = physical_path
        .replace("\\\\.\\PhysicalDrive", "")
        .parse()
        .map_err(|_| "Не вдалось розпізнати номер диску".to_string())?;

    let diskpart_script = format!(
        "select disk {}\nclean\nconvert gpt\ncreate partition primary\nformat fs=ntfs quick\nassign letter={}\n",
        disk_number, drive_letter
    );

    let script_path = "C:\\Temp\\diskpart_script.txt";
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vhd() {
        let path = "C:\\Temp\\test_pocketspace.vhd";
        let result = create_vhd(path, 1);
        assert!(result.is_ok(), "VHD не створився: {:?}", result);
        println!("VHD успішно створено: {}", path);
    }

    #[test]
    fn test_mount_and_detach() {
        let path = "C:\\Temp\\test_pocketspace.vhd";
        
        let handle = mount_and_get_handle(path).expect("Монтування не вдалось");
        println!("VHD змонтовано");

        let detach_result = detach_with_handle(handle);
        assert!(detach_result.is_ok(), "Демонтування не вдалось: {:?}", detach_result);
        println!("VHD демонтовано");
    }

    #[test]
    fn test_mount_and_pause() {
        let path = "C:\\Temp\\test_pocketspace.vhd";
        
        let _handle = mount_and_get_handle(path).expect("Монтування не вдалось");
        println!("VHD змонтовано. Перевір Get-Disk в іншому терміналі.");
        println!("Чекаю 30 секунд...");

        std::thread::sleep(std::time::Duration::from_secs(30));

        println!("Час вийшов, тест завершується (диск автоматично демонтується).");
    }

    #[test]
    fn test_get_physical_path() {
        let path = "C:\\Temp\\test_pocketspace.vhd";
        
        let handle = mount_and_get_handle(path).expect("Монтування не вдалось");
        println!("VHD змонтовано");

        let physical_path = get_physical_path(handle).expect("Не вдалось отримати шлях");
        println!("Фізичний шлях диску: {}", physical_path);

        let detach_result = detach_with_handle(handle);
        assert!(detach_result.is_ok());
        println!("VHD демонтовано");
    }

    #[test]
    fn test_format_vhd() {
        let path = "C:\\Temp\\test_pocketspace.vhd";
        
        let handle = mount_and_get_handle(path).expect("Монтування не вдалось");
        println!("VHD змонтовано");

        let format_result = format_vhd(handle, 'Z');
        assert!(format_result.is_ok(), "Форматування не вдалось: {:?}", format_result);
        println!("Диск відформатовано і призначено букву Z:");

        println!("Перевір зараз провідник Windows! Чекаю 20 секунд...");
        std::thread::sleep(std::time::Duration::from_secs(20));

        let _ = detach_with_handle(handle);
        println!("VHD демонтовано");
    }

    #[test]
    fn test_persistence() {
        let path = "C:\\Temp\\test_pocketspace.vhd";
        
        // Перше підключення — записуємо файл
        let handle1 = mount_and_get_handle(path).expect("Перше монтування не вдалось");
        println!("VHD змонтовано (1-й раз)");

        let test_file_path = "Z:\\test_file.txt";
        std::fs::write(test_file_path, "PocketSpace persistence test")
            .expect("Не вдалось записати файл");
        println!("Файл записано на Z:\\");

        std::thread::sleep(std::time::Duration::from_secs(1));

        let _ = detach_with_handle(handle1);
        println!("VHD демонтовано");

        std::thread::sleep(std::time::Duration::from_secs(2));

        // Друге підключення — перевіряємо файл
        let handle2 = mount_and_get_handle(path).expect("Друге монтування не вдалось");
        println!("VHD змонтовано (2-й раз)");

        std::thread::sleep(std::time::Duration::from_secs(1));

        let content = std::fs::read_to_string(test_file_path)
            .expect("Не вдалось прочитати файл — дані не збереглись!");

        assert_eq!(content, "PocketSpace persistence test");
        println!("Файл прочитано успішно: {}", content);
        println!("ПЕРСИСТЕНТНІСТЬ ПІДТВЕРДЖЕНА");

        let _ = detach_with_handle(handle2);
        println!("VHD демонтовано остаточно");
    }

    #[test]
    fn test_create_large_vhd() {
        let path = "C:\\Temp\\test_encryption.vhd";
        let result = create_vhd(path, 5);
        assert!(result.is_ok(), "VHD не створився: {:?}", result);
        println!("Великий VHD створено: {}", path);
    }

    #[test]
    fn test_create_partitions() {
        let path = "C:\\Temp\\test_encryption.vhd";
        
        let handle = mount_and_get_handle(path).expect("Монтування не вдалось");
        println!("VHD змонтовано");

        let physical_path = get_physical_path(handle).expect("Не вдалось отримати шлях");
        let disk_number: u32 = physical_path
            .replace("\\\\.\\PhysicalDrive", "")
            .parse()
            .expect("Не вдалось розпізнати номер диску");
        println!("Номер диску: {}", disk_number);

        let result = crate::create_two_partitions(disk_number, 2, 'Y', 'X');
        assert!(result.is_ok(), "Розбивка не вдалась: {:?}", result);
        println!("Диск розбито на два розділи: Y: (2GB) і X: (решта)");

        println!("Перевір провідник! Чекаю 20 секунд...");
        std::thread::sleep(std::time::Duration::from_secs(20));

        let _ = detach_with_handle(handle);
        println!("VHD демонтовано");
    }
}