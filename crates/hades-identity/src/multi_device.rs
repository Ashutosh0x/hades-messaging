use std::collections::HashMap;

/// Manages multiple devices for a single user identity.
pub struct DeviceManager {
    primary_device_id: u32,
    linked_devices: HashMap<u32, DeviceStatus>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeviceStatus {
    Active,
    Revoked,
}

impl DeviceManager {
    pub fn new(primary_device_id: u32) -> Self {
        let mut manager = Self {
            primary_device_id,
            linked_devices: HashMap::new(),
        };
        manager.linked_devices.insert(primary_device_id, DeviceStatus::Active);
        manager
    }

    pub fn add_device(&mut self, device_id: u32) {
        self.linked_devices.insert(device_id, DeviceStatus::Active);
    }

    pub fn revoke_device(&mut self, device_id: u32) {
        if device_id != self.primary_device_id {
            self.linked_devices.insert(device_id, DeviceStatus::Revoked);
        }
    }

    pub fn is_active(&self, device_id: u32) -> bool {
        self.linked_devices.get(&device_id) == Some(&DeviceStatus::Active)
    }
}
