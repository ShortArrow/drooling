//! Wire-format parsing for the Pico SDK reset interface requests.
//!
//! Pure functions, host-testable.

/// Parse the `wValue` of a `RESET_REQUEST_BOOTSEL` class request into the
/// argument pair for the boot ROM's `reset_to_usb_boot(gpio_activity_pin_mask,
/// disable_interface_mask)`.
///
/// `wValue` layout (Pico SDK `pico_stdio_usb` reset interface):
/// - bits 0-6: interface disable mask
/// - bit 8: a GPIO activity pin is specified
/// - bits 9-14: the GPIO pin number (when bit 8 is set)
pub fn bootsel_reset_args(w_value: u16) -> (u32, u32) {
    let disable_interface_mask = (w_value & 0x7F) as u32;

    let gpio_specified = (w_value & (1 << 8)) != 0;
    let gpio_activity = if gpio_specified {
        ((w_value >> 9) & 0x3F) as u32
    } else {
        0
    };

    (gpio_activity, disable_interface_mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_gpio_no_disable_mask() {
        assert_eq!(bootsel_reset_args(0x0000), (0, 0));
    }

    #[test]
    fn disable_interface_mask_passes_through() {
        assert_eq!(bootsel_reset_args(0x0003), (0, 0x03));
    }

    #[test]
    fn gpio_bit_unset_ignores_pin_field() {
        assert_eq!(bootsel_reset_args(25 << 9), (0, 0));
    }

    #[test]
    fn gpio_pin_25_with_specified_bit() {
        let w_value = (1 << 8) | (25 << 9);
        assert_eq!(bootsel_reset_args(w_value), (25, 0));
    }
}
