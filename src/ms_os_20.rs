//! Microsoft OS 2.0 Descriptors
//!
//! Implements Windows-specific descriptors required for automatic WinUSB driver loading.

/// MS OS 2.0 Platform Capability UUID
/// {D8DD60DF-4589-4CC7-9CD2-659D9E648A9F}
pub const MS_OS_20_PLATFORM_UUID: [u8; 16] = [
    0xDF, 0x60, 0xDD, 0xD8, 0x89, 0x45, 0xC7, 0x4C,
    0x9C, 0xD2, 0x65, 0x9D, 0x9E, 0x64, 0x8A, 0x9F,
];

/// Vendor code for MS OS 2.0 descriptor requests
pub const MS_OS_20_VENDOR_CODE: u8 = 1;

/// MS OS 2.0 Descriptor Set Total Length
pub const MS_OS_20_DESC_SET_LEN: u16 = 174;

/// BOS (Binary Object Store) Descriptor
///
/// Total length: 5 (header) + 28 (platform capability) = 33 bytes
pub const BOS_DESCRIPTOR: [u8; 33] = [
    // BOS Descriptor Header (5 bytes)
    5,      // bLength
    0x0F,   // bDescriptorType: BOS
    33, 0,  // wTotalLength: 33 bytes (little-endian)
    1,      // bNumDeviceCaps: 1 capability

    // MS OS 2.0 Platform Capability Descriptor (28 bytes)
    28,     // bLength
    0x10,   // bDescriptorType: Device Capability
    0x05,   // bDevCapabilityType: Platform
    0x00,   // bReserved

    // MS OS 2.0 Platform Capability UUID (16 bytes)
    0xDF, 0x60, 0xDD, 0xD8, 0x89, 0x45, 0xC7, 0x4C,
    0x9C, 0xD2, 0x65, 0x9D, 0x9E, 0x64, 0x8A, 0x9F,

    // dwWindowsVersion: 0x06030000 (Windows 8.1 and later)
    0x00, 0x00, 0x03, 0x06,

    // wMSOSDescriptorSetTotalLength: 174 bytes
    0xAE, 0x00,

    // bMS_VendorCode: 1 (used in vendor request)
    MS_OS_20_VENDOR_CODE,

    // bAltEnumCode: 0
    0x00,
];

/// MS OS 2.0 Descriptor Set (174 bytes)
///
/// Provides WinUSB compatible ID and device interface GUID
/// for automatic driver loading on Windows.
pub const MS_OS_20_DESCRIPTOR_SET: [u8; 174] = [
    // Microsoft OS 2.0 descriptor set header (10 bytes)
    0x0A, 0x00,             // wLength: 10
    0x00, 0x00,             // wDescriptorType: MS_OS_20_SET_HEADER_DESCRIPTOR
    0x00, 0x00, 0x03, 0x06, // dwWindowsVersion: 0x06030000 (Windows 8.1)
    0xAE, 0x00,             // wTotalLength: 174

    // Microsoft OS 2.0 configuration subset header (8 bytes)
    // wTotalLength includes this header: 8 + 156 (function subset) = 164
    0x08, 0x00,             // wLength: 8
    0x01, 0x00,             // wDescriptorType: MS_OS_20_SUBSET_HEADER_CONFIGURATION
    0x00,                   // bConfigurationValue: 0 (first config)
    0x00,                   // bReserved
    0xA4, 0x00,             // wTotalLength: 164

    // Microsoft OS 2.0 function subset header (8 bytes)
    // wSubsetLength includes this header: 8 + 20 (compatible ID) + 128 (registry property) = 156
    0x08, 0x00,             // wLength: 8
    0x02, 0x00,             // wDescriptorType: MS_OS_20_SUBSET_HEADER_FUNCTION
    0x00,                   // bFirstInterface: 0 (will be updated based on actual interface)
    0x00,                   // bReserved
    0x9C, 0x00,             // wSubsetLength: 156

    // Microsoft OS 2.0 compatible ID descriptor (20 bytes)
    0x14, 0x00,             // wLength: 20
    0x03, 0x00,             // wDescriptorType: MS_OS_20_FEATURE_COMPATIBLE_ID

    // Compatible ID: "WINUSB\0\0" (8 bytes)
    b'W', b'I', b'N', b'U', b'S', b'B', 0x00, 0x00,

    // Sub-compatible ID: empty (8 bytes)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

    // Microsoft OS 2.0 registry property descriptor (128 bytes)
    0x80, 0x00,             // wLength: 128
    0x04, 0x00,             // wDescriptorType: MS_OS_20_FEATURE_REG_PROPERTY
    0x01, 0x00,             // wPropertyDataType: REG_SZ (Unicode string)
    0x28, 0x00,             // wPropertyNameLength: 40

    // Property name: "DeviceInterfaceGUID" in UTF-16LE (40 bytes)
    b'D', 0, b'e', 0, b'v', 0, b'i', 0, b'c', 0, b'e', 0,
    b'I', 0, b'n', 0, b't', 0, b'e', 0, b'r', 0, b'f', 0,
    b'a', 0, b'c', 0, b'e', 0, b'G', 0, b'U', 0, b'I', 0,
    b'D', 0, 0x00, 0x00,

    0x4E, 0x00,             // wPropertyDataLength: 78

    // Property value: "{bc7398c1-73cd-4cb7-98b8-913a8fca7bf6}" in UTF-16LE (78 bytes)
    b'{', 0, b'b', 0, b'c', 0, b'7', 0, b'3', 0, b'9', 0,
    b'8', 0, b'c', 0, b'1', 0, b'-', 0, b'7', 0, b'3', 0,
    b'c', 0, b'd', 0, b'-', 0, b'4', 0, b'c', 0, b'b', 0,
    b'7', 0, b'-', 0, b'9', 0, b'8', 0, b'b', 0, b'8', 0,
    b'-', 0, b'9', 0, b'1', 0, b'3', 0, b'a', 0, b'8', 0,
    b'f', 0, b'c', 0, b'a', 0, b'7', 0, b'b', 0, b'f', 0,
    b'6', 0, b'}', 0, 0x00, 0x00,
];

/// bFirstInterface offset: set header (10) + configuration subset (8)
/// + function subset wLength (2) + wDescriptorType (2) = 22
pub const FUNCTION_SUBSET_FIRST_INTERFACE_OFFSET: usize = 22;

/// Update MS OS 2.0 descriptor set with correct interface number
pub fn update_interface_number(interface_num: u8) -> [u8; 174] {
    let mut desc = MS_OS_20_DESCRIPTOR_SET;
    desc[FUNCTION_SUBSET_FIRST_INTERFACE_OFFSET] = interface_num;
    desc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u16_at(desc: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes([desc[offset], desc[offset + 1]])
    }

    /// Windows の MS OS 2.0 バリデータと同じ規則でセット全体を走査する。
    /// 各 subset 長は自身のヘッダを含み、子孫は親領域に収まらなければならない。
    #[test]
    fn descriptor_set_lengths_are_consistent() {
        let desc = MS_OS_20_DESCRIPTOR_SET;

        assert_eq!(u16_at(&desc, 0), 10, "set header wLength");
        assert_eq!(u16_at(&desc, 2), 0x00, "set header wDescriptorType");
        assert_eq!(u16_at(&desc, 8) as usize, desc.len(), "set total length");

        let config_start = 10;
        assert_eq!(u16_at(&desc, config_start), 8, "config subset wLength");
        assert_eq!(u16_at(&desc, config_start + 2), 0x01, "config subset type");
        let config_total = u16_at(&desc, config_start + 6) as usize;
        assert_eq!(config_start + config_total, desc.len(), "config subset spans rest of set");

        let func_start = config_start + 8;
        assert_eq!(u16_at(&desc, func_start), 8, "function subset wLength");
        assert_eq!(u16_at(&desc, func_start + 2), 0x02, "function subset type");
        let func_total = u16_at(&desc, func_start + 6) as usize;
        assert_eq!(func_start + func_total, desc.len(), "function subset spans rest of config subset");

        let compat_start = func_start + 8;
        assert_eq!(u16_at(&desc, compat_start), 20, "compatible ID wLength");
        assert_eq!(u16_at(&desc, compat_start + 2), 0x03, "compatible ID type");
        assert_eq!(&desc[compat_start + 4..compat_start + 12], b"WINUSB\0\0");

        let reg_start = compat_start + 20;
        let reg_len = u16_at(&desc, reg_start) as usize;
        assert_eq!(u16_at(&desc, reg_start + 2), 0x04, "registry property type");
        assert_eq!(reg_start + reg_len, desc.len(), "registry property ends exactly at set end");

        let name_len = u16_at(&desc, reg_start + 6) as usize;
        let data_len_off = reg_start + 8 + name_len;
        let data_len = u16_at(&desc, data_len_off) as usize;
        assert_eq!(reg_len, 8 + name_len + 2 + data_len, "registry property internal lengths");
    }

    #[test]
    fn bos_platform_capability_references_set_length() {
        assert_eq!(BOS_DESCRIPTOR.len(), 33);
        assert_eq!(u16_at(&BOS_DESCRIPTOR, 2) as usize, BOS_DESCRIPTOR.len(), "BOS wTotalLength");
        let set_len_in_bos = u16_at(&BOS_DESCRIPTOR, 29) as usize;
        assert_eq!(set_len_in_bos, MS_OS_20_DESCRIPTOR_SET.len(), "wMSOSDescriptorSetTotalLength");
    }

    #[test]
    fn update_interface_number_patches_function_subset() {
        let patched = update_interface_number(2);
        assert_eq!(patched[FUNCTION_SUBSET_FIRST_INTERFACE_OFFSET], 2);
        assert_eq!(u16_at(&patched, 26), 20, "compatible ID wLength must stay intact");
    }
}
