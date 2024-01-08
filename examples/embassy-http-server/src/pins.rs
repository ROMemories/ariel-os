use embassy_nrf::peripherals;
use riot_rs::assign_resources;

// #[cfg(feature = "board0")]
assign_resources! {
    AssignedResources,
    button: Button {
        button: P0_11,
    }
}

#[cfg(feature = "board1")]
assign_resources! {
    AssignedResources,
    button: Button {
        button: P0_12,
    }
}
