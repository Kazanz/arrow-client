// Copyright 2017 click2stream, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;
use std::io;

use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};

use config::ApplicationConfig;

use net::arrow::proto::{ScanReport, Service};
use net::raw::ether::MacAddr;

use svc_table::SharedServiceTableRef;

use utils;

use utils::RuntimeError;

use utils::logger::{BoxedLogger, Severity};

use native_tls::TlsConnector;

use uuid::Uuid;

/// Arrow service connection state.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ConnectionState {
    Connected,
    Disconnected,
    Unauthorized,
}

impl ConnectionState {
    /// Get string representation of the state.
    fn as_str(&self) -> &str {
        match self {
            &ConnectionState::Connected    => "connected",
            &ConnectionState::Disconnected => "disconnected",
            &ConnectionState::Unauthorized => "unauthorized",
        }
    }
}

impl Display for ConnectionState {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.as_str())
    }
}

/// Internal data of the application context.
struct ApplicationContextData {
    logger:      BoxedLogger,
    config:      ApplicationConfig,
    scanning:    bool,
    scan_report: ScanReport,
    conn_state:  ConnectionState,
}

impl ApplicationContextData {
    /// Take a given application config and create application context data.
    fn new(config: ApplicationConfig) -> ApplicationContextData {
        ApplicationContextData {
            logger:      config.get_logger(),
            config:      config,
            scanning:    false,
            scan_report: ScanReport::new(),
            conn_state:  ConnectionState::Disconnected,
        }
    }

    /// Get current version of the configuration.
    fn get_config_version(&self) -> usize {
        self.config.get_version()
    }

    /// Get Arrow Client UUID.
    fn get_arrow_uuid(&self) -> Uuid {
        self.config.get_uuid()
    }

    /// Get Arrow Client password.
    fn get_arrow_password(&self) -> Uuid {
        self.config.get_password()
    }

    /// Get Arrow Client MAC address.
    fn get_arrow_mac_address(&self) -> MacAddr {
        self.config.get_mac_address()
    }

    /// Check if the application is in the diagnostic mode.
    fn get_diagnostic_mode(&self) -> bool {
        self.config.get_diagnostic_mode()
    }

    /// Get application logger.
    fn get_logger(&self) -> BoxedLogger {
        self.logger.clone()
    }

    /// Get TLS connector.
    fn get_tls_connector(&self) -> Result<TlsConnector, RuntimeError> {
        self.config.get_tls_connector()
    }

    /// Set the state of the network scanner thread.
    fn set_scanning(&mut self, scanning: bool) {
        self.scanning = scanning;
    }

    /// Check if the network scanner thread is running right now.
    fn is_scanning(&self) -> bool {
        self.scanning
    }

    /// Get the last scan report.
    fn get_scan_report(&self) -> ScanReport {
        self.scan_report.clone()
    }

    /// Update the scan report.
    fn update_scan_report(&mut self, report: ScanReport) {
        self.scan_report = report;
    }

    /// Get read-only reference to the service table.
    fn get_service_table(&self) -> SharedServiceTableRef {
        self.config.get_service_table()
    }

    /// Reset service table.
    fn reset_service_table(&mut self) {
        self.config.reset_service_table()
    }

    /// Set connection state.
    fn set_connection_state(&mut self, state: ConnectionState) {
        self.conn_state = state;

        let res = self.save_connection_state();

        utils::result_or_log(
            &mut self.logger,
            Severity::WARN,
            "unable to save current connection state",
            res);
    }

    /// Save connection state into the file.
    fn save_connection_state(&self) -> Result<(), io::Error> {
        let mut file = File::create(self.config.get_connection_state_file())?;

        writeln!(&mut file, "{}", self.conn_state)?;

        Ok(())
    }

    /// Add given services into the service table.
    fn update_services<I>(&mut self, services: I)
        where I: IntoIterator<Item=Service> {
        self.config.update_services(services)
    }
}

/// Application context.
#[derive(Clone)]
pub struct ApplicationContext {
    data: Arc<Mutex<ApplicationContextData>>,
}

impl ApplicationContext {
    /// Take a given application config and create a new application context.
    pub fn new(config: ApplicationConfig) -> ApplicationContext {
        ApplicationContext {
            data: Arc::new(Mutex::new(ApplicationContextData::new(config)))
        }
    }

    /// Get current version of the configuration.
    pub fn get_config_version(&self) -> usize {
        self.data.lock()
            .unwrap()
            .get_config_version()
    }

    /// Get Arrow Client UUID.
    pub fn get_arrow_uuid(&self) -> Uuid {
        self.data.lock()
            .unwrap()
            .get_arrow_uuid()
    }

    /// Get Arrow Client password.
    pub fn get_arrow_password(&self) -> Uuid {
        self.data.lock()
            .unwrap()
            .get_arrow_password()
    }

    /// Get Arrow Client MAC address.
    pub fn get_arrow_mac_address(&self) -> MacAddr {
        self.data.lock()
            .unwrap()
            .get_arrow_mac_address()
    }

    /// Check if the application is in the diagnostic mode.
    pub fn get_diagnostic_mode(&self) -> bool {
        self.data.lock()
            .unwrap()
            .get_diagnostic_mode()
    }

    /// Get application logger.
    pub fn get_logger(&self) -> BoxedLogger {
        self.data.lock()
            .unwrap()
            .get_logger()
    }

    /// Get TLS connector.
    pub fn get_tls_connector(&self) -> Result<TlsConnector, RuntimeError> {
        self.data.lock()
            .unwrap()
            .get_tls_connector()
    }

    /// Set the state of the network scanner thread.
    pub fn set_scanning(&mut self, scanning: bool) {
        self.data.lock()
            .unwrap()
            .set_scanning(scanning)
    }

    /// Check if the network scanner thread is running right now.
    pub fn is_scanning(&self) -> bool {
        self.data.lock()
            .unwrap()
            .is_scanning()
    }

    /// Get the last scan report.
    pub fn get_scan_report(&self) -> ScanReport {
        self.data.lock()
            .unwrap()
            .get_scan_report()
    }

    /// Update the scan report.
    pub fn update_scan_report(&mut self, report: ScanReport) {
        self.data.lock()
            .unwrap()
            .update_scan_report(report)
    }

    /// Get read-only reference to the service table.
    pub fn get_service_table(&self) -> SharedServiceTableRef {
        self.data.lock()
            .unwrap()
            .get_service_table()
    }

    /// Reset service table.
    pub fn reset_service_table(&mut self) {
        self.data.lock()
            .unwrap()
            .reset_service_table()
    }

    /// Set connection state.
    pub fn set_connection_state(&mut self, state: ConnectionState) {
        self.data.lock()
            .unwrap()
            .set_connection_state(state)
    }

    /// Add given services into the service table.
    pub fn update_services<I>(&mut self, services: I)
        where I: IntoIterator<Item=Service> {
        self.data.lock()
            .unwrap()
            .update_services(services)
    }
}