use std::process::exit;
use gtk::gio;
use gtk::gio::{BusNameOwnerFlags, BusNameWatcherFlags, BusType, DBusConnection, DBusMessage, DBusMethodInvocation, DBusNodeInfo, DBusSendMessageFlags, OwnerId};
use gtk::glib::{MainLoop, Variant, VariantTy};
use crate::consts::*;
use crate::gio::glib::Sender;

const INTROSPECTION_XML: &str = "\
<node>\
  <interface name='wm.crab.GDBus.LauncherInterface'>\
    <method name='ShowWindow'/>\
  </interface>\
</node>";

pub struct CrabDaemonServer;

impl CrabDaemonServer {
    pub fn new() -> Self {
        Self
    }

    pub fn start(&self, tx: Sender<bool>) -> OwnerId {
        let own_name = gio::bus_own_name(
            BusType::Session,
            DBUS_SESSION_NAME,
            BusNameOwnerFlags::NONE,
            move |conn, name| Self::on_bus_acquired(conn, name, tx.clone()),
            Self::on_name_acquired,
            Self::on_name_lost,
        );

        own_name
    }

    fn handle_method_call(_connection: DBusConnection, _sender: &str, _object_path: &str, _interface_name: &str, method_name: &str, _parameters: Variant, _invocation: DBusMethodInvocation, tx: Sender<bool>) {
        match method_name {
            "ShowWindow" => {
                tx.send(true).unwrap();
            }
            _ => {}
        }
    }

    fn handle_get_property(_connection: DBusConnection, _sender: &str, _object_path: &str, _interface_name: &str, _property_name: &str) -> Variant {
        Variant::from_none(VariantTy::ANY)
    }

    fn handle_set_property(_connection: DBusConnection, _sender: &str, _object_path: &str, _interface_name: &str, _property_name: &str, _value: Variant) -> bool {
        true
    }

    fn on_bus_acquired(connection: DBusConnection, _name: &str, tx: Sender<bool>) {
        let introspection_data = DBusNodeInfo::for_xml(INTROSPECTION_XML).unwrap();

        let _registration_id = connection.register_object(
            DBUS_OBJECT_PATH,
            &introspection_data.lookup_interface(DBUS_INTERFACE_NAME).unwrap(),
            move |connection, sender, object_path, interface_name, method_name, parameters, invocation| Self::handle_method_call(connection, sender, object_path, interface_name, method_name, parameters, invocation, tx.clone()),
            Self::handle_get_property,
            Self::handle_set_property
        ).unwrap();
    }

    fn on_name_acquired(_connection: DBusConnection, _name: &str) {}

    fn on_name_lost(_connection: Option<DBusConnection>, _name: &str) {
        exit(1);
    }
}

pub struct CrabDaemonClient;

impl CrabDaemonClient {
    pub fn new() -> Self {
        Self
    }

    pub fn run_method(&self, method_name: &'static str) {
        let watcher_id = gio::bus_watch_name(
            BusType::Session,
            DBUS_SESSION_NAME,
            BusNameWatcherFlags::NONE,
            |connection, name, name_owner| {
                Self::on_name_appeared(connection, name, name_owner, method_name);
            },
            Self::on_name_vanished
        );

        MainLoop::new(None, false).run();

        gio::bus_unwatch_name(watcher_id);
    }

    fn on_name_appeared(connection: DBusConnection, _name: &str, name_owner: &str, method_name: &str) {
        let method_call_message = DBusMessage::new_method_call(
            Some(name_owner),
            DBUS_OBJECT_PATH,
            Some(DBUS_INTERFACE_NAME),
            method_name
        );

        connection.send_message(&method_call_message, DBusSendMessageFlags::NONE).unwrap();

        exit(0);
    }

    fn on_name_vanished(_connection: DBusConnection, _name: &str) {
        exit(1);
    }
}