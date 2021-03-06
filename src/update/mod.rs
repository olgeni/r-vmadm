//! Update for a jail
use jail_config::{JailConfig, NIC};
use std::error::Error;
use std::io::Read;
use serde_json;
use uuid::Uuid;


macro_rules! update {
    ( $src:ident, $target:ident; $($field:ident),+)  => (
        $(
            match $src.$field {
                Some(ref value) => $target.$field = value.clone(),
                _ => ()
            }
        )*
    );
}
macro_rules! update_option {
    ( $src:ident, $target:ident; $($field:ident),+)  => (
        $(
            match $src.$field {
                Some(ref value) => $target.$field = Some(value.clone()),
                _ => ()
            }
        )*
    );
}


/// update the nics
#[derive(Debug, Deserialize, Clone)]
struct NICUpdate {
    mac: String,
    nic_tag: Option<String>,
    ip: Option<String>,
    gateway: Option<String>,
    netmask: Option<String>,
    vlan: Option<u16>,
    primary: Option<bool>,
    mtu: Option<u32>,
    network_uuid: Option<Uuid>,
}

impl NICUpdate {
    #[cfg(test)]
    pub fn empty(mac: String) -> Self {
        NICUpdate{
            mac,
            nic_tag: None,
            ip: None,
            gateway: None,
            netmask: None,
            vlan: None,
            primary: None,
            mtu: None,
            network_uuid: None,
        }
    }
    pub fn apply(&self, nic: NIC) -> NIC {

        if nic.mac != self.mac {
            return nic
        };

        let mut nic = nic.clone();
        update!(self, nic;
                nic_tag,
                ip,
                netmask,
                gateway,
                primary
        );
        update_option!(self, nic;
                       vlan,
                       mtu,
                       network_uuid
        );
        return nic;
    }
}

/// Jail update
#[derive(Debug, Deserialize, Clone)]
pub struct JailUpdate {
    /// readable alias for the jail
    alias: Option<String>,
    /// hostname of the jail
    hostname: Option<String>,
    /// weather to start this jail on --startup
    autostart: Option<bool>,
    /// max physical memory in MB (memoryuse)
    max_physical_memory: Option<u64>,
    /// mac cpu usage 100 = 1 core (pcpu)
    cpu_cap: Option<u64>,
    /// max quota (zfs quota)
    //    quota: u64,
    /// SysV shared memory size, in bytes (shmsize)
    max_shm_memory: Option<u64>,

    /// locked memory (memorylocked)
    max_locked_memory: Option<u64>,

    /// maximum number of porocesses (maxproc)
    max_lwps: Option<u64>,

    // Metadata fields w/o effect on vmadm at the moment
    archive_on_delete: Option<bool>,
    billing_id: Option<Uuid>,
    do_not_inventory: Option<bool>,
    // Currently has no effect
    dns_domain: Option<String>,

    owner_uuid: Option<Uuid>,
    package_name: Option<String>,
    package_version: Option<String>,
    #[serde(default = "empty_nics")]
    add_nics: Vec<NIC>,
    #[serde(default = "empty_macs")]
    remove_nics: Vec<String>,
    #[serde(default = "empty_nic_update")]
    update_nics: Vec<NICUpdate>,

}

impl JailUpdate {
    /// Reads the config from a reader
    pub fn from_reader<R>(reader: R) -> Result<Self, Box<Error>>
    where
        R: Read,
    {
        let update: JailUpdate = serde_json::from_reader(reader)?;
        return Ok(update);
    }
    #[cfg(test)]
    pub fn empty() -> Self {
        JailUpdate {
            alias: None,
            hostname: None,
            autostart: None,
            max_physical_memory: None,
            cpu_cap: None,
            max_shm_memory: None,
            max_locked_memory: None,
            max_lwps: None,
            archive_on_delete: None,
            billing_id: None,
            do_not_inventory: None,
            dns_domain: None,
            owner_uuid: None,
            package_name: None,
            package_version: None,
            add_nics: vec![],
            remove_nics: vec![],
            update_nics: vec![],

        }
    }
    pub fn apply(&self, config: JailConfig) -> JailConfig {
        let mut c = config.clone();
        update!(self, c;
            autostart,
            alias,
            hostname,
            max_physical_memory,
            cpu_cap,
            max_lwps,
            dns_domain
        );
        update_option!(self, c;
            max_shm_memory,
            max_locked_memory,
            archive_on_delete,
            billing_id,
            do_not_inventory,
            owner_uuid,
            package_name,
            package_version
        );


        c.nics.retain(|nic| !self.remove_nics.contains(&nic.mac));
        for nic in self.add_nics.iter() {
            c.nics.push(nic.clone());
        }
        for update in self.update_nics.iter() {

            c.nics = match update.primary {
                Some(true) =>
                    c.nics.iter().map(|nic| {
                        let mut nic = nic.clone();
                        nic.primary = false;
                        update.apply(nic)
                    }).collect(),
                _ => c.nics.iter().map(|nic| update.apply(nic.clone())).collect()
            };



        }

        return c;
    }
}



fn empty_macs() -> Vec<String> {
    Vec::new()
}

fn empty_nic_update() -> Vec<NICUpdate> {
    Vec::new()
}


fn empty_nics() -> Vec<NIC> {
    Vec::new()
}


#[cfg(test)]
mod tests {
    use jail_config::JailConfig;
    use update::*;
    use uuid::Uuid;

    fn nic00() -> NIC {
        NIC{
            interface: String::from("net0"),
            mac: String::from("00:00:00:00:00:00"),
            vlan: None,
            nic_tag: String::from("admin"),
            ip: String::from("192.168.254.254"),
            netmask: String::from("255.255.255.0"),
            gateway: String::from("192.168.254.1"),
            primary: true,
            mtu: None,
            network_uuid: None
        }
    }
    fn nic01() -> NIC {
        NIC{
            interface: String::from("net0"),
            mac: String::from("00:00:00:00:00:01"),
            vlan: None,
            nic_tag: String::from("admin"),
            ip: String::from("192.168.254.253"),
            netmask: String::from("255.255.255.0"),
            gateway: String::from("192.168.254.1"),
            primary: false,
            mtu: None,
            network_uuid: None
        }
    }
    fn nic02() -> NIC {
        NIC{
            interface: String::from("net0"),
            mac: String::from("00:00:00:00:00:02"),
            vlan: None,
            nic_tag: String::from("admin"),
            ip: String::from("192.168.254.252"),
            netmask: String::from("255.255.255.0"),
            gateway: String::from("192.168.254.1"),
            primary: false,
            mtu: None,
            network_uuid: None
        }
    }

    fn conf() -> JailConfig {
        JailConfig{
            brand: String::from("jail"),
            uuid: Uuid::nil(),
            image_uuid: Uuid::nil(),
            alias: String::from("test-alias"),
            hostname: String::from("test-hostname"),
            autostart: true,
            max_physical_memory: 1024,
            cpu_cap: 100,
            quota: 5,
            max_shm_memory: None,
            max_locked_memory: None,
            nics: vec![nic00(), nic01()],
            max_lwps: 2000,
            archive_on_delete: None,
            billing_id: None,
            do_not_inventory: None,
            dns_domain: String::from("local"),
            indestructible_delegated: None,
            indestructible_zoneroot: None,
            owner_uuid: None,
            package_name: None,
            package_version: None,
        }
    }

    fn uuid() -> Uuid {
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8,
                     9, 10, 11, 12, 13, 14, 15, 16];
        Uuid::from_bytes(&bytes).unwrap()
    }
    #[test]
    fn empty() {
        let conf = conf();
        let update = JailUpdate::empty();
        let conf1 = update.apply(conf.clone());
        assert_eq!(conf, conf1);
    }
    #[test]
    fn alias() {
        let conf = conf();
        let mut update = JailUpdate::empty();
        let alias = String::from("changed");
        update.alias = Some(alias.clone());
        assert_eq!(alias, update.apply(conf).alias);
    }
    #[test]
    fn hostname() {
        let conf = conf();
        let mut update = JailUpdate::empty();
        let hostname = String::from("changed");
        update.hostname = Some(hostname.clone());
        assert_eq!(hostname, update.apply(conf).hostname);
    }
    #[test]
    fn autostart() {
        let conf = conf();
        assert_eq!(true, conf.autostart);
        let mut update = JailUpdate::empty();
        update.autostart = Some(false);
        assert_eq!(false, update.apply(conf).autostart);
    }
    #[test]
    fn max_physical_memory() {
        let conf = conf();
        assert_eq!(1024, conf.max_physical_memory);
        let mut update = JailUpdate::empty();
        update.max_physical_memory = Some(42);
        assert_eq!(42, update.apply(conf).max_physical_memory);
    }
    #[test]
    fn max_locked_memory() {
        let conf = conf();
        assert_eq!(None, conf.max_locked_memory);
        let mut update = JailUpdate::empty();
        update.max_locked_memory = Some(42);
        assert_eq!(42, update.apply(conf).max_locked_memory.unwrap());
    }
    #[test]
    fn max_lwps() {
        let conf = conf();
        assert_eq!(2000, conf.max_lwps);
        let mut update = JailUpdate::empty();
        update.max_lwps = Some(42);
        assert_eq!(42, update.apply(conf).max_lwps);
    }
    #[test]
    fn archive_on_delete() {
        let conf = conf();
        assert_eq!(None, conf.archive_on_delete);
        let mut update = JailUpdate::empty();
        update.archive_on_delete = Some(true);
        assert_eq!(true, update.apply(conf).archive_on_delete.unwrap());
    }
    #[test]
    fn billing_id() {
        let conf = conf();
        assert_eq!(None, conf.billing_id);
        let mut update = JailUpdate::empty();
        update.billing_id = Some(uuid());
        assert_eq!(uuid(), update.apply(conf).billing_id.unwrap());
    }
    #[test]
    fn no_not_inventory() {
        let conf = conf();
        assert_eq!(None, conf.do_not_inventory);
        let mut update = JailUpdate::empty();
        update.do_not_inventory = Some(true);
        assert_eq!(true, update.apply(conf).do_not_inventory.unwrap());
    }
    #[test]
    fn dns_domain() {
        let conf = conf();
        let mut update = JailUpdate::empty();
        let dns_domain = String::from("changed");
        update.dns_domain = Some(dns_domain.clone());
        assert_eq!(dns_domain, update.apply(conf).dns_domain);
    }
    #[test]
    fn owner_uuid() {
        let conf = conf();
        assert_eq!(None, conf.owner_uuid);
        let mut update = JailUpdate::empty();
        update.owner_uuid = Some(uuid());
        assert_eq!(uuid(), update.apply(conf).owner_uuid.unwrap());
    }
    #[test]
    fn package_name() {
        let conf = conf();
        let mut update = JailUpdate::empty();
        let package_name = String::from("changed");
        update.package_name = Some(package_name.clone());
        assert_eq!(package_name, update.apply(conf).package_name.unwrap());
    }
    #[test]
    fn package_version() {
        let conf = conf();
        let mut update = JailUpdate::empty();
        let package_version = String::from("changed");
        update.package_version = Some(package_version.clone());
        assert_eq!(package_version, update.apply(conf).package_version.unwrap());
    }

    #[test]
    fn remove_nics() {
        let conf = conf();
        let mut update = JailUpdate::empty();
        let mac = String::from("00:00:00:00:00:00");
        update.remove_nics = vec![mac];
        assert_eq!(vec![nic01()], update.apply(conf).nics);
    }
    #[test]
    fn add_nics() {
        let conf = conf();
        let mut update = JailUpdate::empty();
        update.add_nics = vec![nic02()];
        assert_eq!(vec![nic00(), nic01(), nic02()], update.apply(conf).nics);
    }

    #[test]
    fn nics_change_primary() {
        let conf = conf();
        let mut update = JailUpdate::empty();
        let mut nic_update = NICUpdate::empty(nic01().mac.clone());
        nic_update.primary = Some(true);
        update.update_nics = vec![nic_update];
        let conf1 = update.apply(conf.clone());

        assert_eq!(false, conf1.nics[0].primary);
        assert_eq!(true, conf1.nics[1].primary);
    }

    // nic update tests

    #[test]
    fn nic_no_update_on_wrong_mac() {
        let nic = nic01();
        let mut update = NICUpdate::empty(nic02().mac.clone());
        let nic_tag = String::from("changed");
        update.nic_tag = Some(nic_tag.clone());
        assert_eq!(nic.clone(), update.apply(nic));
    }

    #[test]
    fn nic_tag() {
        let nic = nic01();
        let mut update = NICUpdate::empty(nic.mac.clone());
        let nic_tag = String::from("changed");
        update.nic_tag = Some(nic_tag.clone());
        assert_eq!(nic_tag, update.apply(nic).nic_tag);
    }

    #[test]
    fn nic_ip() {
        let nic = nic01();
        let mut update = NICUpdate::empty(nic.mac.clone());
        let ip = String::from("192.168.1.254");
        update.ip = Some(ip.clone());
        assert_eq!(ip, update.apply(nic).ip);
    }
    #[test]
    fn nic_gateway() {
        let nic = nic01();
        let mut update = NICUpdate::empty(nic.mac.clone());
        let gateway = String::from("192.168.1.1");
        update.gateway = Some(gateway.clone());
        assert_eq!(gateway, update.apply(nic).gateway);
    }
    #[test]
    fn nic_netmask() {
        let nic = nic01();
        let mut update = NICUpdate::empty(nic.mac.clone());
        let netmask = String::from("255.255.0.0");
        update.netmask = Some(netmask.clone());
        assert_eq!(netmask, update.apply(nic).netmask);
    }
    #[test]
    fn nic_vlan() {
        let nic = nic01();
        let mut update = NICUpdate::empty(nic.mac.clone());
        let vlan = 42;
        update.vlan = Some(vlan);
        assert_eq!(vlan, update.apply(nic).vlan.unwrap());
    }
    #[test]
    fn nic_primary() {
        let nic = nic01();
        let mut update = NICUpdate::empty(nic.mac.clone());
        let primary = true;
        update.primary = Some(primary);
        assert_eq!(primary, update.apply(nic).primary);
    }
    #[test]
    fn nic_mtu() {
        let nic = nic01();
        let mut update = NICUpdate::empty(nic.mac.clone());
        let mtu = 42;
        update.mtu = Some(mtu);
        assert_eq!(mtu, update.apply(nic).mtu.unwrap());
    }

}

