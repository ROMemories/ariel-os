//! Tools and traits for describing device identities.
//!
//! See `ariel_os::identity` for general documentation; that module also represents the public parts
//! of this API.
#![deny(missing_docs)]

/// Trait describing the unique identifier available on a device.
///
/// See the module level documentation on the characteristics of the identifier.
///
/// # Evolution
///
/// In its current state, this type is mainly a wrapper around a binary identifier.
///
/// As it is used more, additional methods can be provided for concrete types of identifiers, such
/// as MAC addresses. By default, those would be generated in some way from what is available in
/// the identifier -- but devices where the identifier already *is* a MAC address (or possibly a
/// range thereof) can provide their official addresses.
pub trait DeviceId: Sized {
    /// Some `[u8; N]` type, returned by [`.bytes()`][Self::bytes].
    ///
    /// This may not represent all the identifying information available on the device, but can
    /// represent a unique portion thereof.
    ///
    /// (For example, if a device has two consecutive MAC addresses assigned, the type as a whole
    /// may represent both, but the conventional serialized identity of the device may just be one
    /// of them).
    ///
    /// # Evolution
    ///
    /// In the long run, it will be preferable to add a `const BYTES_LEN: usize;` and enforce the
    /// type `[u8; Self::BYTES_LEN]` as the return value of [`.bytes(_)]`][Self::bytes]. This can
    /// not be done yet as it depends on the `generic_const_exprs` feature.
    type Bytes: AsRef<[u8]>;

    /// Obtains a unique identifier of the device.
    ///
    /// # Errors
    ///
    /// This produces an error if no device ID is available on this board, or is not implemented.
    /// It is encouraged to use [`core::convert::Infallible`] where possible.
    fn get() -> Result<Self, impl core::error::Error>;

    /// The device identifier in serialized bytes format.
    fn bytes(&self) -> Self::Bytes;

    /// Generates an EUI-48 identifier ("6-byte MAC address") based on the device identity.
    ///
    /// See `ariel_os::identity::interface_eu48` for details.
    fn interface_eui48(&self, if_index: u32) -> [u8; 6] {
        // Not even trying to hash for privacy: Many CPU IDs just have 32 variable bits (eg. EFM32
        // with a 32bit timstamp in a limited range, and a 32bit factory ID, or STM32's 96 bit
        // containing lot and wafer numbers and coordinates), and all SHA256 hashes of 2^32
        // possibilities can just be calculated on a graphics card in an hour.
        //
        // We do hash the board identifier, just to be sure to have a nice and random-looking
        // starting point.

        // FIXME: Can't depend on ariel_os due to cyclic dependencies.
        const BOARD: &str = ariel_os_utils::str_from_env_or!(
            "CONFIG_BOARD",
            "unknown",
            "board name provided by the build system"
        );
        const BOARD_HASH: [u8; 20] = const_sha1::sha1(BOARD.as_bytes()).as_bytes();
        // Once feature(const_option) is stable in our compiler, the unwrap and dereference can
        // move in too.
        let mut eui48: [u8; 6] = *const { BOARD_HASH.first_chunk() }.unwrap();

        // We use the whole device identity to be sure to use the variable parts even if they are
        // just in some location (which may not be the first or the last bits of the bytes(), in
        // general).
        //
        // We make it influence the middle 4 bytes of the MAC address: This is easy to do, and
        // ensures that consecutive serial numbers (no matter in which byte) don't cause chip N
        // interface 1 to have the same MAC as chip N+1 interface 0. Thus we miss the opportunity
        // to influence 12 more bits (out of the 16, because 4 are fixed by construction of an
        // Administratively Assigned Identifier), but the simplicity makes up for it, and whoever
        // runs a risk of having a realistic chance of a MAC collision in 32 bit space will use
        // globally unique addresses or actually manage addresses anyway.

        let bytes = self.bytes();

        // This alternative algorithm is identical (as easily evidenced by running both on
        // arbitrary inputs) but rustc doesn't optimize this simple version:
        //
        // ```
        // for (index, byte) in bytes.as_ref().into_iter().enumerate() {
        //     eui48[1 + index % 4] ^= byte;
        // }
        // ```

        // This would work the same in either little and big endian, but most machines are little
        // these days (and Rust has no simple and safe host-endianness conversion).
        let mut xor_me: u32 = u32::from_le_bytes(eui48[1..5].try_into().unwrap());
        for chunk in bytes.as_ref().chunks(4) {
            let mut full = [0; 4];
            #[allow(
                clippy::indexing_slicing,
                reason = "Works by construction; the equivalent array chunks based construction with accessing the `.remainder` is neither stable nor equally concise"
            )]
            full[..chunk.len()].copy_from_slice(chunk);
            xor_me ^= u32::from_le_bytes(full);
        }
        eui48[1..5].copy_from_slice(&xor_me.to_le_bytes()[..]);

        // Enforce the `?2-??-??-??-??-??` pattern of an AII (Administratively Assigned Identifier)
        eui48[0] &= 0xf0;
        eui48[0] |= 0x02;

        // There is some optimization potential in making everything above this line into a
        // function and inlining the rest, because then constant propagation may eliminate this
        // unaligned thing, but let's delay that until someone measures it.

        let with_if_index =
            u32::from_be_bytes(eui48[2..6].try_into().unwrap()).wrapping_add(if_index);
        eui48[2..6].copy_from_slice(&(with_if_index).to_be_bytes()[..]);

        eui48
    }
}

/// An uninhabited type implementing [`DeviceId`] that always errs.
///
/// This can be used both on architectures that do not have a unique identifier on their boards,
/// and when it has not yet been implemented.
///
/// Typical types for `E` are [`NotImplemented`] or [`NotAvailable`].
#[derive(Debug)]
pub struct NoDeviceId<E: core::error::Error + Default>(
    core::convert::Infallible,
    core::marker::PhantomData<E>,
);

impl<E: core::error::Error + Default> DeviceId for NoDeviceId<E> {
    // We could also come up with a custom never type that AsRef's into [u8], but that won't fly
    // once there is a BYTES_LEN.
    type Bytes = [u8; 0];

    fn get() -> Result<Self, impl core::error::Error> {
        Err::<_, E>(Default::default())
    }

    fn bytes(&self) -> [u8; 0] {
        match self.0 {}
    }
}

/// Error indicating that a [`DeviceId`] may be available on this device, but is not implemented.
#[derive(Debug, Default)]
pub struct NotImplemented;

impl core::fmt::Display for NotImplemented {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Device ID not implemented on this device")
    }
}

impl core::error::Error for NotImplemented {}

/// Error indicating that a [`DeviceId`] is not available on this device.
#[derive(Debug, Default)]
pub struct NotAvailable;

impl core::fmt::Display for NotAvailable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Device ID not available on this device")
    }
}

impl core::error::Error for NotAvailable {}
