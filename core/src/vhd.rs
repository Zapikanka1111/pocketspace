use windows::{
    core::HSTRING,
    Win32::Foundation::{HANDLE, NO_ERROR},
    Win32::Storage::Vhd::{
        CreateVirtualDisk, OpenVirtualDisk, AttachVirtualDisk, DetachVirtualDisk,
        CREATE_VIRTUAL_DISK_FLAG_NONE,
        CREATE_VIRTUAL_DISK_PARAMETERS, OPEN_VIRTUAL_DISK_PARAMETERS,
        OPEN_VIRTUAL_DISK_FLAG, ATTACH_VIRTUAL_DISK_FLAG, DETACH_VIRTUAL_DISK_FLAG,
        OPEN_VIRTUAL_DISK_VERSION_2,
        VIRTUAL_DISK_ACCESS_ALL, VIRTUAL_DISK_ACCESS_NONE,
        VIRTUAL_STORAGE_TYPE_DEVICE_VHD, VIRTUAL_STORAGE_TYPE,
        VIRTUAL_STORAGE_TYPE_VENDOR_MICROSOFT,
    },
};

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
}